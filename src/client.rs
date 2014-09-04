use std::{io, str};

use constants::{CommandMessage, Nick, User};
use message::Message;

pub struct ClientBuilder {
    nick: String,
    pass: Option<String>,
    realname: String,
    username: String,

    host: String,
    port: u16,
}

pub struct Client {
    conn: io::BufferedStream<io::TcpStream>,
}

impl ClientBuilder {
    pub fn new (nick: &str, host: &str) -> ClientBuilder {
        ClientBuilder {
            nick: nick.to_string(),
            pass: None,
            realname: nick.to_string(),
            username: nick.to_string(),

            host: host.to_string(),
            port: 6667,
        }
    }

    pub fn set_pass (&mut self, pass: &str) -> &mut ClientBuilder {
        self.pass = Some(pass.to_string());
        self
    }

    pub fn set_username (&mut self, username: &str) -> &mut ClientBuilder {
        self.username = username.to_string();
        self
    }

    pub fn set_realname (&mut self, realname: &str) -> &mut ClientBuilder {
        self.realname = realname.to_string();
        self
    }

    pub fn set_port (&mut self, port: u16) -> &mut ClientBuilder {
        self.port = port;
        self
    }

    pub fn connect (&mut self) -> Client {
        let mut client = self.connect_raw();
        client.write(
            Message::new(
                None,
                CommandMessage(Nick),
                vec![self.nick.clone()],
            )
        );
        client.write(
            Message::new(
                None,
                CommandMessage(User),
                vec![
                    self.username.clone(),
                    "localhost".to_string(), // XXX
                    self.host.clone(),
                    self.realname.clone(),
                ],
            )
        );
        client
    }

    pub fn connect_raw (&mut self) -> Client {
        let mut stream = io::TcpStream::connect(self.host.as_slice(), self.port);
        Client { conn: io::BufferedStream::new(stream.unwrap()) }
    }
}

impl Client {
    pub fn read (&mut self) -> Message {
        // \n isn't valid inside a message, so this should be fine. if the \n
        // we find isn't preceded by a \r, this will be caught by the message
        // parser.
        let buf = self.conn.read_until(b'\n');
        // XXX handle different encodings
        // XXX proper error handling
        Message::parse(str::from_utf8(buf.unwrap().as_slice()).unwrap()).unwrap()
    }

    pub fn write (&mut self, msg: Message) {
        msg.write_protocol_string(&mut self.conn);
    }
}
