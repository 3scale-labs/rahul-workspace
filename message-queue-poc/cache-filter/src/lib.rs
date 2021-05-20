use log::debug;
use proxy_wasm::{
    traits::*,
    hostcalls::{register_shared_queue, enqueue_shared_queue},
    types::{Action, LogLevel,ContextType},
};

const QUEUE_NAME: &str = "message_queue"; // Note: Name of the queue can be set in the static config as well

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Debug);
    proxy_wasm::set_root_context(|context_id| -> Box<dyn RootContext> {
        Box::new(CacheFilterRoot {context_id, queue_id: None})
    });
    proxy_wasm::set_http_context(|context_id, _: u32| -> Box<dyn HttpContext> {
        Box::new(CacheFilter {context_id})
    });
}

struct CacheFilterRoot {
    context_id: u32,
    queue_id: Option<u32>,
}

struct CacheFilter {
    context_id: u32,
}

impl Context for CacheFilterRoot {}
impl Context for CacheFilter {}

impl RootContext for CacheFilterRoot {
    fn on_vm_start(&mut self, _: usize) -> bool {
        debug!("Registering message queue");
        if let Ok(q_id) = register_shared_queue(QUEUE_NAME) { self.queue_id = Some(q_id); }
        debug!("Completed registering new message queue with id: {}", self.queue_id.unwrap());
        true
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}


impl HttpContext for CacheFilter {
    fn on_http_request_headers(&mut self, _: usize) -> Action {
        debug!("New Incoming Request");
        // We should use resolve here but it requires VM_ID which is not known where it's available.
        let queue_id = register_shared_queue(QUEUE_NAME).unwrap();
        for (name, value) in &self.get_http_request_headers() {
            debug!("In WASM : #{} -> Adding {}: {} to the message queue", self.context_id, name, value);
            match enqueue_shared_queue(queue_id, Some(value.as_bytes())) {
                Ok(_t) => debug!("Enqued: '{}' on Queue with id: {} successfully",value,queue_id),
                Err(e) => debug!("Enquing queue with id: {} failed due to: {:?}",queue_id,e)
            }
        }
    Action::Continue
    }
}