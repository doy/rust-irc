extern crate irc;

use std::collections::HashMap;
use std::io;

pub struct ExampleClient {
    karma: HashMap<String, i32>,
}

impl ExampleClient {
    pub fn new () -> ExampleClient {
        ExampleClient { karma: HashMap::new() }
    }
}

impl irc::ClientCallbacks for ExampleClient {
    fn on_rpl_welcome (&mut self, client: &mut irc::Client, _m: &irc::Message) -> io::IoResult<()> {
        client.join(["#doytest"], [])
    }

    fn on_privmsg (&mut self, client: &mut irc::Client, from: Option<&str>, receivers: &[&str], text: &str) -> io::IoResult<()> {
        let incr = if text.ends_with("++") { 1 }
            else if text.ends_with("--") { -1 }
            else { 0 };
        if incr != 0 {
            let text = text.slice(0, text.len() - 2);
            let giving_user = String::from_utf8(
                from.unwrap().bytes().take_while(|&c| c != b'!').collect()
            ).unwrap();
            if giving_user.as_slice() == text {
                try!(client.notice(receivers[0], "You can't give karma to yourself!"));
            }
            else {
                self.karma.insert_or_update_with(
                    text.to_string(),
                    incr,
                    |_k, v| { *v += incr }
                );
            }
        }
        else if text.starts_with("karma ") {
            let text = text.slice_from(6);
            let default = 0;
            let karma = self.karma.find(&text.to_string()).unwrap_or(&default);
            try!(client.notice(receivers[0], format!("Karma for {} is {}", text, karma).as_slice()));
        }

        Ok(())
    }
}

fn main () {
    let mut builder = irc::ClientBuilder::new("doytest", "chat.freenode.net");
    builder.set_debug(true);
    let client = builder.connect();
    client.run_loop_with_callbacks(ExampleClient::new());
}
