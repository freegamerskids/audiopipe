use std::sync::{Arc, Mutex};

use cpal::traits::DeviceTrait;
use iced_node_editor::LogicalEndpoint;

use crate::NodeEvents;

pub struct InputDevice {
    next_function: Arc<Mutex<Box<dyn FnMut(Vec<i16>) + Send + 'static>>>,

    stream: Option<cpal::Stream>,
}

impl InputDevice {
    pub fn new(device: cpal::Device, next_function: Arc<Mutex<Box<dyn FnMut(Vec<i16>) + Send + 'static>>>) -> Self {
        let next_fn = Arc::clone(&next_function);
        let stream = create_stream(device, next_fn);

        InputDevice {
            next_function,
            stream: Some(stream)
        }
    }
}

impl NodeEvents for InputDevice {
    fn on_connect(&mut self, _start:&LogicalEndpoint, _end: &LogicalEndpoint) {}

    fn on_disconnect(&mut self, last_connection:bool, _start:&LogicalEndpoint, _end: &LogicalEndpoint) {
        if last_connection {
            self.stream = None;
        }
    }

    fn on_data(&mut self, _data: &[i16]) {}
}

fn create_stream(device: cpal::Device, next_function: Arc<Mutex<Box<dyn FnMut(Vec<i16>) + Send + 'static>>>) -> cpal::Stream {
    let config: cpal::StreamConfig = device.default_input_config().unwrap().into();

    let next_fn = Arc::clone(&next_function);

    let input_data_fn = move |data: &[i16], _: &cpal::InputCallbackInfo| {
        if let Ok(mut next_fn) = next_fn.lock() {
            (next_fn)(data.to_vec());
        }
    };

    let stream = device.build_input_stream::<i16, _, _>(&config, input_data_fn, err_fn, None).unwrap();
    stream
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}