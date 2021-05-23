use log::debug;
use proxy_wasm::traits::{Context, RootContext};
use std::time::Duration;
use proxy_wasm::types::LogLevel;
use proxy_wasm::hostcalls::{
    register_shared_queue,
    dequeue_shared_queue
};


const QUEUE_NAME: &str = "message_queue"; // Note: Name of the queue can be set in the static config as well
const PUBLISH_TIME: u64 = 30; // This time should be sufficient so that all requests are processed
#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Debug);
    proxy_wasm::set_root_context(|root_context_id| -> Box<dyn RootContext> {
        Box::new(SingletonService {root_context_id, queue_id: None, enqueue_time: 0.0,  enqueue_count: 0})
    });
}

struct SingletonService {
    root_context_id: u32,
    queue_id: Option<u32>,
    enqueue_time: f64,
    enqueue_count: u32,
}

impl Context for SingletonService {}

impl RootContext for SingletonService {

    fn on_vm_start(&mut self, _vm_configuration_size: usize) -> bool {
        // Register MessageQ to receive info from cache filters
        if let Ok(q_id) = register_shared_queue(QUEUE_NAME) { self.queue_id = Some(q_id); }
        debug!("Registered new message queue with id: {} with context id: {}", self.queue_id.unwrap(),self.root_context_id);
        self.set_tick_period(Duration::from_secs(PUBLISH_TIME));
        true
    }

    // As soon as message queue is ready, message is consumed from it.
    fn on_queue_ready(&mut self, _queue_id: u32) {
        
        match dequeue_shared_queue(self.queue_id.unwrap()) {
            Ok(message_res) => {
                if let Some(message_bytes) = message_res {
                    let message = String::from_utf8(message_bytes).unwrap();
                    if message.contains("0.00") {
                        self.enqueue_time = self.enqueue_time + message.parse::<f64>().unwrap();
                        self.enqueue_count = self.enqueue_count + 1;
                    }
                    debug!("RCID#{}: Consumed following message from the shared queue: {}", self.root_context_id,message);
                }
            },
            Err(e) => debug!("Consuming message queue failed due to: {:?}",e)
        }
    }

    // After every tick duration, stats are published on terminal and set to initial values
    fn on_tick(&mut self) {
        debug!("Reporting enquing stats: Total time: {} Total Count: {}", self.enqueue_time, self.enqueue_count);
        self.enqueue_time = 0.0;
        self.enqueue_count = 0;
    }
}