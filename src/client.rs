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
        let mut stream = io::TcpStream::connect(self.servername.as_slice(), self.port);
        let mut stream = stream.unwrap();
        let socket_name = match stream.socket_name() {
            Ok(addr) => Some(addr.ip.to_string()),
            Err(_) => None,
        };
        Client::new(self, io::BufferedStream::new(stream), socket_name)
    }
}

pub trait Client {
    fn new (builder: ClientBuilder<Self>, conn: io::BufferedStream<io::TcpStream>, socket_name: Option<String>) -> Self;
    fn builder (&self) -> &ClientBuilder<Self>;
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

    fn run_loop_with (mut self, handler: |&mut Self, Message|) -> Self {
        loop {
            let m = self.read();
            handler(&mut self, m);
        }
        self
    }

    // XXX once storing closures in structs works, we'll also want to provide
    // a default CallbackClient impl of Client to allow users to not have to
    // worry about the struct layout for simple cases.
    fn run_loop (mut self) {
        Client::on_connect(&mut self);

        let mut client = self.run_loop_with(|client, m| {
            Client::on_message(client, m);
        });

        Client::on_disconnect(&mut client);
    }

    fn on_connect (client: &mut Self) {
        let nick = client.builder().nick.clone();
        let pass = client.builder().pass.clone();
        let username = client.builder().username.clone();
        let servername = client.builder().servername.clone();
        let realname = client.builder().realname.clone();

        match pass {
            Some(pass) => {
                client.write(Message::new(None, Pass, vec![pass]));
            },
            None => {},
        }

        client.write(Message::new(None, Nick, vec![nick]));

        let hostname = match client.builder().hostname {
            Some(ref host) => host.clone(),
            None => {
                match client.socket_name() {
                    Some(ref host) => host.to_string(),
                    // XXX something better here?
                    None => "localhost".to_string(),
                }
            },
        };

        client.write(
            Message::new(
                None, User, vec![ username, hostname, servername, realname ],
            )
        );
    }

    fn on_disconnect (client: &mut Self) { }

    fn on_message (client: &mut Self, m: Message) { }
}
