use std::collections::HashMap;

use iced::Point;
use iced_node_editor::Matrix;

use crate::{Application, NodeState, SocketType};

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
                    text: String::from("Iced"),
                    button: false,
                    sockets: (vec![], vec![SocketType::BlueSquare, SocketType::RedCircle]),
                },
                // Node #1
                NodeState {
                    position: Point::new(250.0, 250.0),
                    text: String::from("Node"),
                    button: false,
                    sockets: (
                        vec![SocketType::RedCircle],
                        vec![
                            SocketType::RedCircle,
                            SocketType::BlueSquare,
                            SocketType::Button,
                        ],
                    ),
                },
                // Node #2
                NodeState {
                    position: Point::new(500.0, 250.0),
                    text: String::from("Editor"),
                    button: true,
                    sockets: (vec![SocketType::BlueSquare, SocketType::RedCircle], vec![]),
                },
            ],
            connections,
            dangling_source: None,
            dangling_connection: None,
        }
    }
}