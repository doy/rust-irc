extern crate irc;

use irc::constants::Pong;
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

    fn on_any_message (_client: &mut ExampleClient, m: &irc::Message) {
        print!("{}", m.to_protocol_string());
    }

    fn on_ping (client: &mut ExampleClient, _from: Option<&str>, server1: &str, server2: Option<&str>) {
        let params = match server2 {
            Some(server2) => vec![server1.to_string(), server2.to_string()],
            None => vec![server1.to_string()],
        };
        client.write(irc::Message::new(None, Pong, params));
    }
}

fn main () {
    let builder = irc::ClientBuilder::new("doytest", "chat.freenode.net");
    let client: ExampleClient = builder.connect();
    client.run_loop();
}
