extern crate irc;

fn main () {
    let builder = irc::ClientBuilder::new("doytest", "chat.freenode.net");
    let client = builder.connect();
    client.run_loop_with(|_client, m| {
        println!("{}", m);
    });
}
