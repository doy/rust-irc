use std::{io, str};

use constants::{Nick, Pass, User};
use message::Message;

pub struct ClientBuilder {
    nick: String,
    pass: Option<String>,
    realname: String,
    username: String,

    hostname: Option<String>,

    servername: String,
    port: u16,
}

pub struct Client {
    conn: io::BufferedStream<io::TcpStream>,
    socket_name: Option<String>,
}

impl ClientBuilder {
    pub fn new (nick: &str, servername: &str) -> ClientBuilder {
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

    pub fn set_hostname (&mut self, hostname: &str) -> &mut ClientBuilder {
        self.hostname = Some(hostname.to_string());
        self
    }

    pub fn set_port (&mut self, port: u16) -> &mut ClientBuilder {
        self.port = port;
        self
    }

    pub fn connect (self) -> Client {
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
                    Some(host) => host.to_string(),
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

    pub fn connect_raw (self) -> Client {
        let mut stream = io::TcpStream::connect(self.servername.as_slice(), self.port);
        let mut stream = stream.unwrap();
        let socket_name = match stream.socket_name() {
            Ok(addr) => Some(addr.ip.to_string()),
            Err(_) => None,
        };
        Client {
            conn: io::BufferedStream::new(stream),
            socket_name: socket_name,
        }
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

    pub fn socket_name (&self) -> Option<&str> {
        match self.socket_name {
            Some(ref name) => Some(name.as_slice()),
            None => None,
        }
    }

    // XXX eventually, we'll want to set up callbacks for specific events
    // beforehand, and just have a `run_loop` method that loops and calls the
    // preset callbacks as necessary. unfortunately, rust doesn't handle
    // storing closures very well yet if they need to receive a borrowed
    // pointer, and we would need to pass the client object into the callback
    // in order to make this work
    pub fn run_loop_with (&mut self, handler: |Message|) {
        loop {
            handler(self.read());
        }
    }
}
