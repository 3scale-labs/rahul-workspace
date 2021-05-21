use log::debug;
use proxy_wasm::traits::{Context, RootContext};
use proxy_wasm::types::LogLevel;
use std::{time::Duration, str::from_utf8};
use proxy_wasm::hostcalls::{
    register_shared_queue,
    dequeue_shared_queue
};


const QUEUE_NAME: &str = "message_queue"; // Note: Name of the queue can be set in the static config as well
const TICK_DURATION: Duration = Duration::from_secs(2); // on_tick() will be called every duration unit described here

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Debug);
    proxy_wasm::set_root_context(|root_context_id| -> Box<dyn RootContext> {
        Box::new(SingletonService {root_context_id, queue_id: None})
    });
}

struct SingletonService {
    root_context_id: u32,
    queue_id: Option<u32>
}

impl Context for SingletonService {}

impl RootContext for SingletonService {

    fn on_vm_start(&mut self, _vm_configuration_size: usize) -> bool {
        // Register MessageQ to receive info from cache filters
        if let Ok(q_id) = register_shared_queue(QUEUE_NAME) { self.queue_id = Some(q_id); }
        debug!("Registered new message queue with id: {} with context id: {}", self.queue_id.unwrap(),self.root_context_id);
        self.set_tick_period(TICK_DURATION);
        true
    }

    // Every tick duration, message is consumed from shared queue.
    fn on_tick(&mut self) {
        
        match dequeue_shared_queue(self.queue_id.unwrap()) {
            Ok(message_res) => {
                if let Some(message_bytes) = message_res {
                    debug!("RCID#{}: Consumed following message from the shared queue: {}", self.root_context_id,from_utf8(&message_bytes).unwrap());
                }
            },
            Err(e) => debug!("Consuming message queue failed due to: {:?}",e)
        }
    }
}