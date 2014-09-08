extern crate irc;

use irc::constants::{Ping, Pong};
use irc::Client;

use std::io;

struct ExampleClient {
    builder: irc::ClientBuilder<ExampleClient>,
    conn: io::BufferedStream<io::TcpStream>,
    socket_name: Option<String>,
}

impl irc::Client for ExampleClient {
    fn new (builder: irc::ClientBuilder<ExampleClient>, conn: io::BufferedStream<io::TcpStream>, socket_name: Option<String>) -> ExampleClient {
        ExampleClient { builder: builder, conn: conn, socket_name: socket_name }
    }
    fn builder (&self) -> &irc::ClientBuilder<ExampleClient> {
        &self.builder
    }
    fn conn (&mut self) -> &mut io::BufferedStream<io::TcpStream> {
        &mut self.conn
    }
    fn socket_name (&self) -> Option<&str> {
        match self.socket_name {
            Some(ref name) => Some(name.as_slice()),
            None => None,
        }
    }

    fn on_message (client: &mut ExampleClient, m: irc::Message) {
        print!("{}", m.to_protocol_string());
        match *m.message_type() {
            Ping => {
                client.write(irc::Message::new(None, Pong, m.params().clone()));
            },
            _ => {},
        }
    }
}

fn main () {
    let builder = irc::ClientBuilder::new("doytest", "chat.freenode.net");
    let client: ExampleClient = builder.connect();
    client.run_loop();
}
