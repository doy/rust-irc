extern crate irc;

fn main () {
    let mut client = irc::ClientBuilder::new("doytest", "chat.freenode.net").connect();

    loop {
        let res = client.read();
        println!("{}", res);
    }
}
