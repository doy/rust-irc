extern crate irc;

fn main () {
    let mut client = irc::ClientBuilder::new("doytest", "chat.freenode.net").connect();
    client.run_loop_with(|m| {
        println!("{}", m);
    });
}
