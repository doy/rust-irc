extern crate irc;

use std::io;

pub struct ExampleClient;

impl irc::ClientCallbacks for ExampleClient {
    fn on_any_message (&mut self, _client: &mut irc::Client, m: &irc::Message) -> io::IoResult<()> {
        print!("{}", m.to_protocol_string());
        Ok(())
    }
}

fn main () {
    let builder = irc::ClientBuilder::new("doytest", "chat.freenode.net");
    let client = builder.connect();
    client.run_loop_with_callbacks(ExampleClient);
}
