use log::debug;
use std::time::SystemTime;
use proxy_wasm::{
    traits::*,
    hostcalls::{resolve_shared_queue, enqueue_shared_queue, get_current_time},
    types::{Action, LogLevel},
};

// These values preferably be configured through envoy.yaml but there is no need for root context so having them here is efficient.
const QUEUE_NAME: &str = "message_queue"; 
const VM_ID: &str = "my_vm_id"; // This should be same as mentioned in the config for singleton service

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Debug);
    proxy_wasm::set_http_context(|context_id, _: u32| -> Box<dyn HttpContext> {
        Box::new(CacheFilter {context_id})
    });
}

struct CacheFilter {
    context_id: u32,
}

impl Context for CacheFilter {}

impl HttpContext for CacheFilter {
    
    fn on_http_request_headers(&mut self, _: usize) -> Action {  
        // Get the queue_id to pass info to the singleton service
        let queue_id = resolve_shared_queue(VM_ID, QUEUE_NAME).unwrap();

        // Benchmarking setup
        let mut total_time = 0.0;
        let mut headers_count = 0;

        // Enquing request headers in Message Queue (dummy data) for testing
        for (name, value) in &self.get_http_request_headers() {
            headers_count = headers_count + 1;
            debug!("In WASM : #{} -> Adding {}: {} to the message queue", self.context_id, name, value);

            //We cannot use Instant::now() because it causes panic on every WASM platform.
            let start_time = get_current_time().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64();

            match enqueue_shared_queue(queue_id.unwrap(), Some(value.as_bytes())) {
                Ok(_t) => debug!("Enqued: '{}' on Queue with id: {} successfully",value,queue_id.unwrap()),
                Err(e) => debug!("Enquing queue with id: {} failed due to: {:?}",queue_id.unwrap(),e)
            }

            let end_time = get_current_time().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64();
            total_time = end_time - start_time;
        }
        
        let average_time = (total_time/headers_count as f64).to_string();
        debug!("Average time to enqueue headers: {} ", average_time);
        if let Ok(_) = enqueue_shared_queue(queue_id.unwrap(), Some(average_time.as_bytes())) {}
    Action::Continue
    }
}