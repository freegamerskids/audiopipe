pub mod window;

use std::collections::HashMap;

use iced::Point;
use iced_node_editor::Matrix;

use crate::{Application, NodeState, NodeAttribute, NodeType};

impl Application {
    pub fn new() -> Self {
        let mut connections = HashMap::new();
        connections.insert((2, 0), (1, 1)); // Output socket #1 of node #1 to input socket #0 of node #2
        connections.insert((1, 0), (0, 1)); // Output socket #1 of node #0 to input socket #0 of node #1

        Application {
            matrix: Matrix::identity(),
            nodes: vec![
                // Node #0
                NodeState {
                    position: Point::new(0.0, 0.0),
                    node_name: String::from("Iced"),
                    node_type: NodeType::Microphone,
                    attributes: (vec![], vec![NodeAttribute::BlueSquare(None), NodeAttribute::RedCircle(None)]),
                    next_function: None
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
                    next_function: None
                },
                // Node #2
                NodeState {
                    position: Point::new(500.0, 250.0),
                    node_name: String::from("Editor"),
                    node_type: NodeType::PlaybackDevice,
                    attributes: (vec![NodeAttribute::BlueSquare(None), NodeAttribute::RedCircle(None)], vec![]),
                    next_function: None
                },
            ],
            connections,
            dangling_source: None,
            dangling_connection: None,
        }
    }
}