extern crate irc;

use irc::constants::{CommandMessage, Nick, User};

fn main () {
    let mut client = irc::Client::new("doytest", "chat.freenode.net", 6667);
    client.connect();

    client.write(
        irc::Message::new(
            None,
            CommandMessage(Nick),
            vec!["doytest".to_string()],
        )
    );
    client.write(
        irc::Message::new(
            None,
            CommandMessage(User),
            vec![
                "doytest".to_string(),
                "localhost".to_string(),
                "localhost".to_string(),
                "doytest".to_string(),
            ],
        )
    );

    loop {
        let res = client.read();
        println!("{}", res);
    }
}
