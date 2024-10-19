use std::sync::{Arc, Mutex};

use iced_node_editor::LogicalEndpoint;

use crate::{platform::platform_impl::audio_capture::ImplAudioCaptureStream, NodeEvents};

pub struct WindowCapture {
    next_function: Arc<Mutex<Box<dyn FnMut(Vec<i16>) + Send + 'static>>>,

    process_id: i32,
    capture_stream: Option<ImplAudioCaptureStream>,
}

impl WindowCapture {
    pub fn new(process_id: i32, next_function: Arc<Mutex<Box<dyn FnMut(Vec<i16>) + Send + 'static>>>) -> Self {
        Self {
            next_function,
            capture_stream: None,
            process_id,
        }
    }
}

impl NodeEvents for WindowCapture {
    fn on_connect(&mut self, _start:&LogicalEndpoint, _end: &LogicalEndpoint) {
        if self.capture_stream.is_none() {
            let next_fn = Arc::clone(&self.next_function);
            let capture_stream = ImplAudioCaptureStream::new(self.process_id, Box::new(move |packet| {
                //println!("packet received");
                if let Ok(mut next_fn) = next_fn.lock() {
                    (next_fn)(packet.unwrap().data().to_vec());
                }
            })).unwrap();

            self.capture_stream = Some(capture_stream);
        }
    }

    fn on_disconnect(&mut self, _start:&LogicalEndpoint, _end: &LogicalEndpoint) {
        println!("on_disconnect");
    }

    fn on_data(&mut self, _data: &[i16]) {}
}