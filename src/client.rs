use std::{io, str};
use std::collections::hashmap::HashMap;

use constants::{MessageType, Nick, Pass, Ping, Pong, User};
use message::Message;

type Callback<'a, 'b> = Box<Fn<(&'a mut Client<'a, 'b>, &'b Message), ()> + 'static>;

#[deriving(PartialEq, Eq, Hash)]
pub enum CallbackEvent {
    MessageEvent(MessageType),
    // XXX timer? connect/disconnect?
}

// XXX these lifetime parameters make no sense at all, but appear to be
// necessary for now...
pub struct ClientBuilder<'a, 'b> {
    nick: String,
    pass: Option<String>,
    realname: String,
    username: String,

    hostname: Option<String>,

    servername: String,
    port: u16,

    callbacks: HashMap<CallbackEvent, Vec<Callback<'a, 'b>>>,
}

pub struct Client<'a, 'b> {
    conn: io::BufferedStream<io::TcpStream>,
    callbacks: HashMap<CallbackEvent, Vec<Callback<'a, 'b>>>,
}

impl<'a, 'b> ClientBuilder<'a, 'b> {
    pub fn new (nick: &str, servername: &str) -> ClientBuilder<'a, 'b> {
        let mut callbacks = HashMap::new();

        callbacks.insert(
            MessageEvent(Ping),
            vec![
                box () (|&: client: &mut Client, m: &Message| {
                    client.write(Message::new(None, Pong, m.params().clone()));
                }) as Callback
            ]
        );

        ClientBuilder {
            nick: nick.to_string(),
            pass: None,
            realname: nick.to_string(),
            username: nick.to_string(),

            hostname: None,

            servername: servername.to_string(),
            port: 6667,

            callbacks: callbacks,
        }
    }

    pub fn set_pass (&mut self, pass: &str) -> &mut ClientBuilder<'a, 'b> {
        self.pass = Some(pass.to_string());
        self
    }

    pub fn set_username (&mut self, username: &str) -> &mut ClientBuilder<'a, 'b> {
        self.username = username.to_string();
        self
    }

    pub fn set_realname (&mut self, realname: &str) -> &mut ClientBuilder<'a, 'b> {
        self.realname = realname.to_string();
        self
    }

    pub fn set_hostname (&mut self, hostname: &str) -> &mut ClientBuilder<'a, 'b> {
        self.hostname = Some(hostname.to_string());
        self
    }

    pub fn set_port (&mut self, port: u16) -> &mut ClientBuilder<'a, 'b> {
        self.port = port;
        self
    }

    pub fn connect (self) -> Client<'a, 'b> {
        let nick = self.nick.clone();
        let pass = self.pass.clone();
        let hostname = match self.hostname {
            Some(ref host) => host.clone(),
            None => {
                // XXX get the name of the local end of the connection
                "localhost".to_string()
            },
        };
        let username = self.username.clone();
        let servername = self.servername.clone();
        let realname = self.realname.clone();

        let mut client = self.connect_raw();

        match pass {
            Some(pass) => {
                client.write(Message::new(None, Pass, vec![pass]));
            },
            None => {},
        }

        client.write(Message::new(None, Nick, vec![nick]));

        client.write(
            Message::new(
                None, User, vec![ username, hostname, servername, realname ],
            )
        );
        client
    }

    pub fn connect_raw (self) -> Client<'a, 'b> {
        let mut stream = io::TcpStream::connect(self.servername.as_slice(), self.port);
        Client {
            conn: io::BufferedStream::new(stream.unwrap()),
            callbacks: self.callbacks,
        }
    }
}

impl<'a, 'b> Client<'a, 'b> {
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
