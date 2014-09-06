extern crate irc;

use irc::constants::{Ping, Pong};

fn main () {
    let builder = irc::ClientBuilder::new("doytest", "chat.freenode.net");
    let client = builder.connect();
    client.run_loop_with(|client, m| {
        println!("{}", m);
        match m.message_type() {
            &Ping => {
                client.write(irc::Message::new(None, Pong, m.params().clone()));
            },
            _ => {},
        }
    });
}
