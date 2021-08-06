use log::{debug, info, warn};
use proxy_wasm::hostcalls::*;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|context_id| -> Box<dyn RootContext> {
        Box::new(FilterRoot { context_id })
    });
}

#[derive(Clone)]
struct Filter {
    context_id: u32,
    cache_key: String,
}

struct FilterRoot {
    context_id: u32,
}

impl Context for FilterRoot {}

impl Context for Filter {
    fn on_http_call_response(&mut self, token_id: u32, _: usize, _: usize, _: usize) {
        info!(
            "ctxt {}: received response: token: {}",
            self.context_id, token_id
        );
        self.resume_http_request();
        self.free_callout_lock(&self.cache_key);
    }
}

thread_local! {
    static WAITING_CONTEXTS: RefCell<HashMap<u32, Filter>> = RefCell::new(HashMap::new());
}

impl HttpContext for Filter {
    fn on_http_request_headers(&mut self, _: usize) -> Action {
        info!("CACHE MISS!");
        // After checking, proxy decides to consider cache miss flow and
        // before making an auth call, it will try to acquire a lock for callout.
        if self.get_callout_lock(&self.cache_key) {
            let hdrs = vec![
                (":authority", "management_service:3001"),
                (":scheme", "http"),
                (":path", "/"),
                (":method", "GET"),
            ];
            match dispatch_http_call(
                "management_service",
                hdrs,
                None,
                vec![],
                Duration::new(5, 0),
            ) {
                Ok(callout_id) => debug!(
                    "ctxt#{}: successfully disptached call: {}",
                    self.context_id, callout_id
                ),
                Err(e) => {
                    warn!("ctxt#{}: failed to dispatch call: {:?}", self.context_id, e);
                    self.send_http_response(500, vec![], Some(b"Internal Failure"));
                    self.free_callout_lock(&self.cache_key);
                }
            }
        } else {
            info!("ctxt#{}: unable to acquire callout-lock", self.context_id);
            WAITING_CONTEXTS.with(|waiters| {
                if waiters
                    .borrow_mut()
                    .insert(self.context_id, self.clone())
                    .is_some()
                {
                    // should not be possible but just in case.
                    warn!(
                        "already waiting for a callout response for this context(id: {})",
                        self.context_id
                    );
                    self.send_http_response(500, vec![], Some(b"Internal Failure"));
                } else {
                    debug!(
                        "successfully added context(id: {}) to the callout wait-list",
                        self.context_id
                    );
                }
            });

            // failed to get the lock so set tick period and wait for the reponse.
            if let Err(e) = set_tick_period(Duration::from_millis(100)) {
                warn!("failed to set tick period: {:?}", e);
                // error due to internal problem.
                self.send_http_response(500, vec![], Some(b"Internal Failure"));
                // TODO: remove context from wait-list
            }
        }
        Action::Pause
    }
}

impl Filter {
    // returns true only when log is acquired
    // Note: Should use Result instead of boolean as return value
    fn get_callout_lock(&self, cache_key: &str) -> bool {
        let request_key = format!("req_{}", cache_key);

        // check if lock is already acquired or not.
        if let Ok((None, cas)) = get_shared_data(&request_key) {
            // log is not acquired by now.
            // we can also add thread id as value for better debugging.
            match set_shared_data(&request_key, Some(b"lock"), cas) {
                Ok(()) => return true,                    // lock acquired
                Err(Status::CasMismatch) => return false, // someone acquired it first
                Err(e) => {
                    warn!("failed to acquire lock due to internal error: {:?}", e);
                    return false;
                }
            }
        }
        false // already acquired by someone and not released yet.
    }

    // returns true on successful lock deletion.
    fn free_callout_lock(&self, cache_key: &str) -> bool {
        let request_key = format!("req_{}", cache_key);
        // assuming only 1 thread makes the callout
        if let Err(e) = set_shared_data(&request_key, None, None) {
            warn!(
                "ctxt#{}: failed to delete the callout-lock for request: {}: {:?}",
                self.context_id, request_key, e
            );
            return false;
        }
        true
    }

    fn handle_cache_hit(&mut self) -> bool {
        info!("CACHE HIT CALLED!");
        self.resume_http_request();
        true
    }
}

impl RootContext for FilterRoot {
    fn on_vm_start(&mut self, _: usize) -> bool {
        debug!("VM started");
        true
    }

    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(Filter {
            context_id,
            cache_key: "common_cache_key".to_string(),
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn on_tick(&mut self) {
        debug!("on_tick called for root-context: {}", self.context_id);
        WAITING_CONTEXTS.with(|waiters| {
            waiters.borrow_mut().retain(|context_id, filter| {
                debug!(
                    "checking callout response for http-context(id: {})",
                    context_id
                );
                let request_key = format!("req_{}", filter.cache_key);
                match get_shared_data(&request_key) {
                    Ok((Some(_), _)) => true, // still waiting for the response.
                    Ok((None, _)) => {
                        // someone deleted the callout-lock and we can proceed with cache hit flow.
                        // NOTE: get app first when writing into main repo
                        set_effective_context(*context_id).unwrap();
                        filter.handle_cache_hit();
                        false
                    }
                    Err(e) => {
                        warn!(
                            "failed to find callout-lock in the shared data for {} : {:?}",
                            request_key, e
                        );
                        set_effective_context(*context_id).unwrap();
                        send_http_response(500, vec![], Some(b"Internal Failure")).unwrap();
                        false
                    }
                }
            });
            if waiters.borrow().is_empty() {
                set_tick_period(Duration::from_millis(0)).unwrap();
            }
        })
    }
}
