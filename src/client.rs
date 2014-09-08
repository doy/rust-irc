use std::{io, str};

use constants::{Nick, Pass, User};
use message::Message;

pub struct ClientBuilder<T: Client> {
    nick: String,
    pass: Option<String>,
    realname: String,
    username: String,

    hostname: Option<String>,

    servername: String,
    port: u16,
}

impl<T: Client> ClientBuilder<T> {
    pub fn new (nick: &str, servername: &str) -> ClientBuilder<T> {
        ClientBuilder {
            nick: nick.to_string(),
            pass: None,
            realname: nick.to_string(),
            username: nick.to_string(),

            hostname: None,

            servername: servername.to_string(),
            port: 6667,
        }
    }

    pub fn set_pass (&mut self, pass: &str) -> &mut ClientBuilder<T> {
        self.pass = Some(pass.to_string());
        self
    }

    pub fn set_username (&mut self, username: &str) -> &mut ClientBuilder<T> {
        self.username = username.to_string();
        self
    }

    pub fn set_realname (&mut self, realname: &str) -> &mut ClientBuilder<T> {
        self.realname = realname.to_string();
        self
    }

    pub fn set_hostname (&mut self, hostname: &str) -> &mut ClientBuilder<T> {
        self.hostname = Some(hostname.to_string());
        self
    }

    pub fn set_port (&mut self, port: u16) -> &mut ClientBuilder<T> {
        self.port = port;
        self
    }

    pub fn connect (self) -> T {
        let nick = self.nick.clone();
        let pass = self.pass.clone();
        let hostname = self.hostname.clone();
        let username = self.username.clone();
        let servername = self.servername.clone();
        let realname = self.realname.clone();

        let mut client = self.connect_raw();

        let hostname = match hostname {
            Some(host) => host,
            None => {
                match client.socket_name() {
                    Some(ref host) => host.to_string(),
                    // XXX something better here?
                    None => "localhost".to_string(),
                }
            },
        };

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

    pub fn connect_raw (self) -> T {
        let mut stream = io::TcpStream::connect(self.servername.as_slice(), self.port);
        let mut stream = stream.unwrap();
        let socket_name = match stream.socket_name() {
            Ok(addr) => Some(addr.ip.to_string()),
            Err(_) => None,
        };
        Client::new(io::BufferedStream::new(stream), socket_name)
    }
}

pub trait Client {
    fn new (conn: io::BufferedStream<io::TcpStream>, socket_name: Option<String>) -> Self;
    fn conn (&mut self) -> &mut io::BufferedStream<io::TcpStream>;
    fn socket_name (&self) -> Option<&str>;

    fn read (&mut self) -> Message {
        // \n isn't valid inside a message, so this should be fine. if the \n
        // we find isn't preceded by a \r, this will be caught by the message
        // parser.
        let buf = self.conn().read_until(b'\n');
        // XXX handle different encodings
        // XXX proper error handling
        Message::parse(str::from_utf8(buf.unwrap().as_slice()).unwrap()).unwrap()
    }

    fn write (&mut self, msg: Message) {
        msg.write_protocol_string(self.conn());
    }

    // XXX eventually, we'll want to set up callbacks for specific events
    // beforehand, and just have a `run_loop` method that loops and calls the
    // preset callbacks as necessary. unfortunately, rust doesn't handle
    // storing closures very well yet if they need to receive a borrowed
    // pointer, and we would need to pass the client object into the callback
    // in order to make this work
    fn run_loop_with (mut self, handler: |&mut Self, Message|) {
        loop {
            let m = self.read();
            handler(&mut self, m);
        }
    }
}
