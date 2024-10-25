use iced::widget::{button, container, text};
use iced::{Color, Element, Length, Padding, Point};
use iced_node_editor::{
    graph_container, node, Connection, Endpoint, GraphNodeElement, Link, LogicalEndpoint,
    Socket, SocketRole, SocketSide,
};

use crate::{Application, NodeAttribute, NodeEvents};

#[derive(Debug, Clone)]
pub enum Message {
    ScaleChanged(f32, f32, f32),
    TranslationChanged(f32, f32),
    MoveNode(usize, f32, f32),
    Connect(Link),
    Disconnect(LogicalEndpoint, Point),
    Dangling(Option<(LogicalEndpoint, Link)>),
    ButtonPressed,
}

impl Application {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ScaleChanged(x, y, scale) => {
                self.matrix = self
                    .matrix
                    .translate(-x, -y)
                    .scale(if scale > 0.0 { 1.05 } else { 1.0 / 1.05 })
                    .translate(x, y);
            }
            Message::TranslationChanged(x, y) => self.matrix = self.matrix.translate(x, y),
            Message::MoveNode(i, x, y) => {
                self.nodes[i].position = Point::new(
                    self.nodes[i].position.x + x / self.matrix.get_scale(),
                    self.nodes[i].position.y + y / self.matrix.get_scale(),
                );
            }
            Message::Connect(link) => {
                // The call to `unwrap_sockets` will panic if the `link` contains absolute
                // endpoints. But the `Connect` message is guaranteed to only contain `Link`s with
                // both endpoints being sockets.
                let (start, end) = link.unwrap_sockets();
                self.nodes[start.node_index].node_type.on_connect(&start, &end);
                self.nodes[end.node_index].node_type.on_connect(&start, &end);

                // Insert the new connection. The hash map design ensures that this will delete any
                // potentially previously present connections ending in the same node.
                self.connections.insert(
                    (end.node_index, end.socket_index),
                    (start.node_index, start.socket_index),
                );
            }
            Message::Disconnect(endpoint, new_dangling_end_position) => {
                // Remove the connection that ends in the socket, if it exists
                if let Some((start_node_index, start_socket_index)) = self
                    .connections
                    .remove(&(endpoint.node_index, endpoint.socket_index))
                {
                    let is_last_connection = self.connections.get(&(endpoint.node_index, endpoint.socket_index)).is_none();

                    let start_endpoint = LogicalEndpoint{node_index: start_node_index, role: SocketRole::Out, socket_index: start_socket_index};
                    self.nodes[start_node_index].node_type.on_disconnect(is_last_connection, &start_endpoint, &endpoint);
                    self.nodes[endpoint.node_index].node_type.on_disconnect(is_last_connection, &start_endpoint, &endpoint);

                    // If there was a connection, turn it into a dangling one, such that the user
                    // may connect it to some other socket instead. First, set the source of the
                    // new dangling connection
                    let new_dangling_source = LogicalEndpoint {
                        node_index: start_node_index,
                        role: SocketRole::Out,
                        socket_index: start_socket_index,
                    };
                    self.dangling_source = Some(new_dangling_source);

                    // Construct a link for the dangling connection.
                    //
                    // This is not necessary just for correct behaviour.
                    // The node editor would emit the `Dangling` event with the full `Link`
                    // as soon as the mouse is moved anyway.
                    // However, if we do not do this, no dangling connection will be drawn until
                    // the mouse is moved. To be able to avoid this slight jank, the library
                    // provides us with a destination point to construct a new dangling connection
                    // for ourselves.
                    self.dangling_connection = Some(Link::from_unordered(
                        Endpoint::Socket(new_dangling_source),
                        Endpoint::Absolute(new_dangling_end_position),
                    ));
                }
            }
            Message::Dangling(Some((source, link))) => {
                // The dangling connection is updated, perhaps because the user moved their mouse
                self.dangling_source = Some(source);
                self.dangling_connection = Some(link);
            }
            Message::Dangling(None) => {
                // The dangling connection is cleared, e.g. when releasing the left mouse button
                // while not hovering over a valid target socket
                self.dangling_source = None;
                self.dangling_connection = None;
            }
            Message::ButtonPressed => println!("Button was pressed."),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut graph_content: Vec<GraphNodeElement<Message, _, _>> = vec![];

        // Convert our own node representations into widgets
        for (i, n) in self.nodes.iter().enumerate() {
            // Create sockets from our lists of `NodeAttribute`s
            let (in_sockets, out_sockets) = &n.attributes;
            let mut node_sockets = vec![];
            for (role, sockets) in [(SocketRole::In, in_sockets), (SocketRole::Out, out_sockets)] {
                for socket_type in sockets {
                    // Call our own utility function to create the socket
                    let new_socket = make_socket(role, socket_type);
                    node_sockets.push(new_socket);
                }
            }

            let node = node(text(&n.node_name));

            graph_content.push(
                node.padding(Padding::from(10.0))
                    .sockets(node_sockets)
                    .center_x()
                    .center_y()
                    .on_translate(move |p| Message::MoveNode(i, p.0, p.1))
                    .width(Length::Fixed(200.0))
                    .height(Length::Fixed(75.0))
                    .position(n.position)
                    .into(),
            );
        }

        // Convert our own `HashMap` representation of connections into the one used by the library.
        // Here it is important that this happens *after* the nodes have been added.
        // The socket layouting logic needs to process first the nodes, then the connections,
        // to have the information necessary to correctly position connection endpoints.
        for ((end_node_index, end_socket_index), (start_node_index, start_socket_index)) in
            self.connections.iter()
        {
            graph_content.push(
                Connection::between(
                    Endpoint::Socket(LogicalEndpoint {
                        node_index: *start_node_index,
                        role: SocketRole::Out,
                        socket_index: *start_socket_index,
                    }),
                    Endpoint::Socket(LogicalEndpoint {
                        node_index: *end_node_index,
                        role: SocketRole::In,
                        socket_index: *end_socket_index,
                    }),
                )
                .into(),
            );
        }

        // Append the dangling connection, if one exists
        if let Some(link) = &self.dangling_connection {
            graph_content.push(Connection::new(link.clone()).into())
        }

        container(
            graph_container(graph_content)
                .dangling_source(self.dangling_source)
                .on_translate(|p| Message::TranslationChanged(p.0, p.1))
                .on_scale(Message::ScaleChanged)
                .on_connect(Message::Connect)
                .on_disconnect(Message::Disconnect)
                .on_dangling(Message::Dangling)
                .width(Length::Fill)
                .height(Length::Fill)
                .matrix(self.matrix),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

fn make_socket<'a>(
    role: SocketRole,
    socket_type: &NodeAttribute,
) -> Socket<'a, Message, iced::Theme, iced::Renderer> {
    // With this, we determine that the input sockets should be on the left side of a node
    // and the output sockets on the right side. The opposite would be possible as well,
    // as would a more complex arrangement where some input and output sockets are on the same side.
    let blob_side = match role {
        SocketRole::In => SocketSide::Left,
        SocketRole::Out => SocketSide::Right,
    };

    // In principle, we could also decouple the alignment of the socket content
    // (which is the element that is displayed within the node at the same height as the blob)
    // from the position of the blob, such that for example, a socket's blob appears on
    // the left side, but its label on the right side.
    // Here, we go with the obvious assignment
    let content_alignment = match role {
        SocketRole::In => iced::alignment::Horizontal::Left,
        SocketRole::Out => iced::alignment::Horizontal::Right,
    };

    const BLOB_RADIUS: f32 = 5.0;

    // The style of the blob is not determined by a style sheet, but by properties of the `Socket`
    // itself.
    let (blob_border_radius, blob_color, content) = match socket_type {
        NodeAttribute::BlueSquare(name) => (
            0.0,
            Color::from_rgb(0.0, 0.1, 0.8),
            text(name.clone().unwrap_or("Blue square".to_string())).into(),
        ),
        NodeAttribute::RedCircle(name) => (
            BLOB_RADIUS,
            Color::from_rgb(0.8, 0.1, 0.0),
            text(name.clone().unwrap_or("Red circle".to_string())).into(),
        ),
        NodeAttribute::Button(name) => (
            BLOB_RADIUS,
            Color::from_rgb(0.3, 0.3, 0.3),
            button(name.clone().unwrap_or("Button")).on_press(Message::ButtonPressed).into(),
        ),
    };

    Socket {
        role,
        blob_side,
        content_alignment,

        blob_radius: BLOB_RADIUS,
        blob_border_radius,
        blob_color,
        content, // Arbitrary widgets can be used here.

        min_height: 0.0,
        max_height: f32::INFINITY,
        blob_border_color: None, // If `None`, the one from the style sheet will be used.
    }
}