mod render;
mod node;
pub mod audio_capture;

use std::collections::HashMap;

use iced::Point;
use iced_node_editor::{Matrix, Link, LogicalEndpoint};

pub struct NodeState {
    position: Point,
    text: String,
    button: bool,
    sockets: (Vec<SocketType>, Vec<SocketType>),
}

// Define some types that sockets may have.
// The library does not perform any sort of type checking; it is entirely up to user code to verify
// that sockets with correct types are connected to each other. In this example, we just use the
// types to provide sockets with two different appearances (that behave identically).
pub enum SocketType {
    BlueSquare,
    RedCircle,
    Button,
}

pub struct Application {
    matrix: Matrix,
    nodes: Vec<NodeState>,

    // Adjacency map of connections: the key corresponds to the node and socket index of the
    // connection **target** — the one on the right of the connection, the *input* socket at
    // which this connection ends. This is the better representation, because disconnections
    // originate from input sockets, and so we can easily look up the connections ending at
    // a certain input socket.
    //
    // For this example, we also make the restriction that only one connection may end in a
    // specific input socket, so it is doubly beneficial because we do not need a `Vec`
    // in the value type.
    connections: HashMap<(usize, usize), (usize, usize)>,

    // Our own representation of the “dangling connection” — the connection that follows the user's
    // mouse pointer in the process of connecting two sockets with each other.
    // It is divided into two parts:
    //  - the `dangling_source` represents the endpoint from which the dangling connection
    //    originates *logically*. It is used to provide correct functionality when connecting nodes.
    //  - the `dangling_connection` is essentially purely aesthetic; it is just an additional
    //    connection that is drawn such that the user gets some feedback on what they are doing.
    dangling_source: Option<LogicalEndpoint>,
    dangling_connection: Option<Link>,
}