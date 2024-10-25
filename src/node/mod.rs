pub mod window;
pub mod playback_device;
pub mod input_device;

use std::collections::HashMap;

use iced::Point;
use iced_node_editor::{LogicalEndpoint, Matrix};
use input_device::InputDevice;

use crate::{Application, NodeAttribute, NodeEvents, NodeState, NodeType};

impl NodeEvents for NodeType {
    fn on_connect(&mut self, start:&LogicalEndpoint, end: &LogicalEndpoint) {
        match self {
            NodeType::InputDevice(input_device) => {
                input_device.on_connect(start, end);
            }
            NodeType::PlaybackDevice(playback_device) => {
                playback_device.on_connect(start, end);
            }
            NodeType::Window(window) => {
                window.on_connect(start, end);
            }
        }
    }

    fn on_disconnect(&mut self, last_connection:bool, start:&LogicalEndpoint, end: &LogicalEndpoint) {
        match self {
            NodeType::InputDevice(input_device) => {
                input_device.on_disconnect(last_connection,start, end);
            }
            NodeType::PlaybackDevice(playback_device) => {
                playback_device.on_disconnect(last_connection, start, end);
            }
            NodeType::Window(window) => {
                window.on_disconnect(last_connection, start, end);
            }
        }
    }

    fn on_data(&mut self, data: &[i16]) {
        match self {
            NodeType::InputDevice(input_device) => {
                input_device.on_data(data);
            }
            NodeType::PlaybackDevice(playback_device) => {
                playback_device.on_data(data);
            }
            NodeType::Window(window) => {
                window.on_data(data);
            }
        }
    }
}

impl Application {
    pub fn new() -> Self {
        let mut connections = HashMap::new();
        connections.insert((2, 0), (1, 1)); // Output socket #1 of node #1 to input socket #0 of node #2
        connections.insert((1, 0), (0, 1)); // Output socket #1 of node #0 to input socket #0 of node #1

        Application {
            matrix: Matrix::identity(),
            nodes: vec![
                /*// Node #0
                NodeState {
                    position: Point::new(0.0, 0.0),
                    node_name: String::from("Iced"),
                    node_type: NodeType::InputDevice(InputDevice{}),
                    attributes: (vec![], vec![NodeAttribute::BlueSquare(None), NodeAttribute::RedCircle(None)]),
                },
                // Node #1
                NodeState {
                    position: Point::new(250.0, 250.0),
                    node_name: String::from("Node"),
                    node_type: NodeType::Microphone,
                    attributes: (
                        vec![NodeAttribute::RedCircle(None)],
                        vec![
                            NodeAttribute::RedCircle(None),
                            NodeAttribute::BlueSquare(Some("aaaaaaa".to_string())),
                            NodeAttribute::Button(Some("I am a button")),
                        ],
                    ),
                },
                // Node #2
                NodeState {
                    position: Point::new(500.0, 250.0),
                    node_name: String::from("Editor"),
                    node_type: NodeType::PlaybackDevice,
                    attributes: (vec![NodeAttribute::BlueSquare(None), NodeAttribute::RedCircle(None)], vec![]),
                },*/
            ],
            connections,
            dangling_source: None,
            dangling_connection: None,
        }
    }
}