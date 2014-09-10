extern crate irc;

pub struct ExampleClient;

impl irc::ClientCallbacks for ExampleClient {
}

fn main () {
    let mut builder = irc::ClientBuilder::new("doytest", "chat.freenode.net");
    builder.set_debug(true);
    let client = builder.connect();
    client.run_loop_with_callbacks(ExampleClient);
}
