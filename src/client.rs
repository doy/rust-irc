use std::{io, str};

use message::Message;

pub struct Client {
    nick: String,
    host: String,
    port: u16,

    connection: Option<io::BufferedStream<io::TcpStream>>,
}

impl Client {
    pub fn new (nick: &str, host: &str, port: u16) -> Client {
        Client {
            nick: nick.to_string(),
            host: host.to_string(),
            port: port,
            connection: None,
        }
    }

    pub fn connect (&mut self) {
        let mut stream = io::TcpStream::connect(self.host.as_slice(), self.port);
        self.connection = Some(io::BufferedStream::new(stream.unwrap()));
    }

    pub fn read (&mut self) -> Message {
        // \n isn't valid inside a message, so this should be fine. if the \n
        // we find isn't preceded by a \r, this will be caught by the message
        // parser.
        match self.connection {
            Some(ref mut conn) => {
                let buf = conn.read_until(b'\n');
                // XXX handle different encodings
                // XXX proper error handling
                Message::parse(str::from_utf8(buf.unwrap().as_slice()).unwrap()).unwrap()
            },
            None => fail!(),
        }
    }
}
