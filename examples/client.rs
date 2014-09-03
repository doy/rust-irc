extern crate irc;

fn main () {
    let mut client = irc::Client::new("doytest", "chat.freenode.net", 6667);
    client.connect();
    loop {
        let res = client.read();
        println!("{}", res);
    }
}
