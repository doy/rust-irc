extern crate irc;

use irc::constants::{Ping, Pong};
use irc::Client;

use std::io;

struct ExampleClient {
    conn: io::BufferedStream<io::TcpStream>,
    socket_name: Option<String>,
}

impl irc::Client for ExampleClient {
    fn new (conn: io::BufferedStream<io::TcpStream>, socket_name: Option<String>) -> ExampleClient {
        ExampleClient { conn: conn, socket_name: socket_name }
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
}

fn main () {
    let builder = irc::ClientBuilder::new("doytest", "chat.freenode.net");
    let client: ExampleClient = builder.connect();
    client.run_loop_with(|client, m| {
        println!("{}", m);
        match *m.message_type() {
            Ping => {
                client.write(irc::Message::new(None, Pong, m.params().clone()));
            },
            _ => {},
        }
    });
}
