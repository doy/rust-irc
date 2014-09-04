extern crate irc;

fn main () {
    let mut client = irc::ClientBuilder::new("doytest", "chat.freenode.net")
        .add_callback(irc::Client::AnyMessageEvent, |client, m| {
            println!("{}", m)
        })
        .connect();
}
