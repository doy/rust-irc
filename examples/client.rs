extern crate irc;

use irc::constants::Pong;

use std::io;

pub struct ExampleClient;

impl irc::ClientCallbacks for ExampleClient {
    fn on_any_message (&mut self, _client: &mut irc::Client, m: &irc::Message) -> io::IoResult<()> {
        print!("{}", m.to_protocol_string());
        Ok(())
    }

    fn on_ping (&mut self, client: &mut irc::Client, _from: Option<&str>, server1: &str, server2: Option<&str>) -> io::IoResult<()> {
        let params = match server2 {
            Some(server2) => vec![server1.to_string(), server2.to_string()],
            None => vec![server1.to_string()],
        };
        client.write(irc::Message::new(None, Pong, params))
    }
}

fn main () {
    let builder = irc::ClientBuilder::new("doytest", "chat.freenode.net");
    let client = builder.connect();
    client.run_loop_with_callbacks(ExampleClient);
}
