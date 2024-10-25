use std::sync::Arc;

use cpal::traits::DeviceTrait;
use iced_node_editor::LogicalEndpoint;
use ringbuf::{
    storage::Heap, traits::{Consumer, Producer, Split}, wrap::caching::Caching, HeapRb, SharedRb
};

use crate::NodeEvents;

const LATENCY: f32 = 150_f32; // ms

pub struct PlaybackDevice {
    producer: Caching<Arc<SharedRb<Heap<i16>>>, true, false>,
    stream: cpal::Stream,
}

impl PlaybackDevice {
    pub fn new(device: cpal::Device) -> Self {
        let config: cpal::StreamConfig = device.default_input_config().unwrap().into();
        let latency_frames = (LATENCY / 1_000.0) * config.sample_rate.0 as f32;
        let latency_samples = latency_frames as usize * config.channels as usize;

        let ring = HeapRb::<i16>::new(latency_samples * 2);
        let (mut producer, mut consumer) = ring.split();

        for _ in 0..latency_samples {
            producer.try_push(0).unwrap();
        }

        let output_data_fn = move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            let mut input_fell_behind = false;
            for sample in data {
                *sample = match consumer.try_pop() {
                    Some(s) => s,
                    None => {
                        input_fell_behind = true;
                        0
                    }
                };
            }
            if input_fell_behind {
                eprintln!("input stream fell behind: try increasing latency");
            }
        };

        let stream = device.build_output_stream::<i16, _, _>(&config, output_data_fn, err_fn, None).unwrap();

        PlaybackDevice {
            producer,
            stream
        }
    }
}

impl NodeEvents for PlaybackDevice {
    fn on_connect(&mut self, _start:&LogicalEndpoint, _end: &LogicalEndpoint) {}

    fn on_disconnect(&mut self, _last_connection:bool, _start:&LogicalEndpoint, _end: &LogicalEndpoint) {
        // OPTIMIZE: stop the stream when the last connection is removed
    }

    fn on_data(&mut self, data: &[i16]) {
        for sample in data {
            self.producer.try_push(*sample).unwrap();
        }
    }
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}