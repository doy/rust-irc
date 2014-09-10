use std::io;

use constants::*;
use message::Message;

pub enum MessageError {
    ParseError(&'static str),
    IoError(io::IoError),
}
pub type MessageResult = Result<Message, MessageError>;

pub struct ClientBuilder {
    nick: String,
    pass: Option<String>,
    realname: String,
    username: String,

    hostname: Option<String>,

    servername: String,
    port: u16,

    debug: bool,
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

            debug: false,
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

    pub fn set_debug (&mut self, debug: bool) -> &mut ClientBuilder {
        self.debug = debug;
        self
    }

    pub fn connect (self) -> Client {
        let stream = io::TcpStream::connect(self.servername.as_slice(), self.port);
        let mut stream = stream.unwrap();
        let socket_name = match stream.socket_name() {
            Ok(addr) => Some(addr.ip.to_string()),
            Err(_) => None,
        };
        Client::new(self, io::BufferedStream::new(stream), socket_name)
    }
}

pub struct Client {
    builder: ClientBuilder,
    conn: io::BufferedStream<io::TcpStream>,
    socket_name: Option<String>,
}

impl Client {
    pub fn new (builder: ClientBuilder, conn: io::BufferedStream<io::TcpStream>, socket_name: Option<String>) -> Client {
        Client { builder: builder, conn: conn, socket_name: socket_name }
    }
    pub fn builder (&self) -> &ClientBuilder {
        &self.builder
    }
    pub fn conn (&mut self) -> &mut io::BufferedStream<io::TcpStream> {
        &mut self.conn
    }
    pub fn socket_name (&self) -> Option<&str> {
        match self.socket_name {
            Some(ref name) => Some(name.as_slice()),
            None => None,
        }
    }

    pub fn read (&mut self) -> MessageResult {
        // \n isn't valid inside a message, so this should be fine. if the \n
        // we find isn't preceded by a \r, this will be caught by the message
        // parser.
        let mut buf = [0, ..MAX_MESSAGE_LENGTH];
        let mut len = 0;
        for (res, i) in self.conn().bytes().zip(range(0, MAX_MESSAGE_LENGTH - 1)) {
            match res {
                Ok(b) => {
                    buf[i] = b;
                    if b == b'\n' {
                        len = i + 1;
                        break;
                    }
                },
                Err(e) => return Err(IoError(e)),
            }
        }

        // XXX handle different encodings
        match Message::parse(String::from_utf8_lossy(buf.slice(0, len)).as_slice()) {
            Ok(m) => {
                if self.builder.debug {
                    print!("R {}", m.to_protocol_string());
                }
                Ok(m)
            },
            Err(s) => Err(ParseError(s)),
        }
    }

    pub fn write (&mut self, m: Message) -> io::IoResult<()> {
        try!(m.write_protocol_string(self.conn()));
        if self.builder.debug {
            print!("W {}", m.to_protocol_string());
        }
        Ok(())
    }

    pub fn run_loop (&mut self, handler: |&mut Client, &Message| -> io::IoResult<()>) -> io::IoError {
        loop {
            let m = match self.read() {
                Ok(m) => m,
                Err(ParseError(_e)) => {
                    // XXX this shouldn't stop the loop, but it's not clear
                    // what it should do - warn maybe?
                    continue
                },
                Err(IoError(e)) => return e,
            };
            match handler(self, &m) {
                Err(e) => return e,
                _ => {},
            }
        }
    }

    pub fn run_loop_with_callbacks<T: ClientCallbacks> (mut self, cbs: T) -> io::IoError {
        cbs.run_loop(&mut self)
    }

    pub fn pass (&mut self, pass: &str) -> io::IoResult<()> {
        self.write(Message::new(None, Pass, vec![pass.to_string()]))
    }
    pub fn nick (&mut self, nick: &str) -> io::IoResult<()> {
        self.write(Message::new(None, Nick, vec![nick.to_string()]))
    }
    pub fn user (&mut self, username: &str, hostname: &str, servername: &str, realname: &str) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            User,
            vec![
                username.to_string(),
                hostname.to_string(),
                servername.to_string(),
                realname.to_string(),
            ]
        ))
    }
    pub fn oper (&mut self, user: &str, pass: &str) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Oper,
            vec![
                user.to_string(),
                pass.to_string(),
            ]
        ))
    }
    pub fn quit (&mut self, msg: Option<&str>) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Quit,
            vec![].append(msg.map(|s| s.to_string()).as_slice())
        ))
    }

    pub fn join (&mut self, channels: &[&str], keys: &[&str]) -> io::IoResult<()> {
        let mut params = vec![channels.connect(",")];
        if keys.len() > 0 {
            params.push(keys.connect(","));
        }
        self.write(Message::new(None, Join, params))
    }
    pub fn part (&mut self, channels: &[&str]) -> io::IoResult<()> {
        self.write(Message::new(None, Part, vec![channels.connect(",")]))
    }
    pub fn channel_mode (&mut self, channel: &str, modes: &str, params: &[&str]) -> io::IoResult<()> {
        let p: Vec<String> = params.iter().map(|s| s.to_string()).collect();
        self.write(Message::new(
            None,
            Mode,
            vec![channel.to_string(), modes.to_string()].append(p.as_slice())
        ))
    }
    pub fn user_mode (&mut self, nickname: &str, modes: &str) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Mode,
            vec![nickname.to_string(), modes.to_string()]
        ))
    }
    pub fn topic (&mut self, channel: &str, topic: Option<&str>) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Topic,
            vec![
                channel.to_string()
            ].append(topic.map(|s| s.to_string()).as_slice())
        ))
    }
    pub fn names (&mut self, channels: &[&str]) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Topic,
            if channels.len() > 0 {
                vec![channels.connect(",")]
            }
            else {
                vec![]
            }
        ))
    }
    pub fn list (&mut self, channels: &[&str], server: Option<&str>) -> io::IoResult<()> {
        let mut params = vec![];
        if channels.len() > 0 {
            params.push(channels.connect(","));
        }
        self.write(Message::new(
            None,
            List,
            params.append(server.map(|s| s.to_string()).as_slice())
        ))
    }
    pub fn invite (&mut self, nickname: &str, channel: &str) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Invite,
            vec![nickname.to_string(), channel.to_string()]
        ))
    }
    pub fn kick (&mut self, channel: &str, user: &str, comment: Option<&str>) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Kick,
            vec![
                channel.to_string(),
                user.to_string(),
            ].append(comment.map(|s| s.to_string()).as_slice())
        ))
    }

    pub fn version (&mut self, server: Option<&str>) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Version,
            vec![].append(server.map(|s| s.to_string()).as_slice())
        ))
    }
    pub fn stats (&mut self, query: Option<&str>, server: Option<&str>) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Stats,
            vec![].append(query.map(|s| s.to_string()).as_slice())
                  .append(server.map(|s| s.to_string()).as_slice())
        ))
    }
    pub fn links (&mut self, remote_server: Option<&str>, server_mask: Option<&str>) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Links,
            vec![].append(remote_server.map(|s| s.to_string()).as_slice())
                  .append(server_mask.map(|s| s.to_string()).as_slice())
        ))
    }
    pub fn time (&mut self, server: Option<&str>) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Time,
            vec![].append(server.map(|s| s.to_string()).as_slice())
        ))
    }
    pub fn connect (&mut self, target_server: &str, port: Option<u16>, remote_server: Option<&str>) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Connect,
            vec![
                target_server.to_string(),
            ].append(port.map(|s| s.to_string()).as_slice())
             .append(remote_server.map(|s| s.to_string()).as_slice())
        ))
    }
    pub fn trace (&mut self, server: Option<&str>) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Trace,
            vec![].append(server.map(|s| s.to_string()).as_slice())
        ))
    }
    pub fn admin (&mut self, server: Option<&str>) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Admin,
            vec![].append(server.map(|s| s.to_string()).as_slice())
        ))
    }
    pub fn info (&mut self, server: Option<&str>) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Info,
            vec![].append(server.map(|s| s.to_string()).as_slice())
        ))
    }

    pub fn privmsg (&mut self, receivers: &[&str], text: &str) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Privmsg,
            vec![
                receivers.connect(","),
                text.to_string(),
            ]
        ))
    }
    pub fn notice (&mut self, nickname: &str, text: &str) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Notice,
            vec![
                nickname.to_string(),
                text.to_string(),
            ]
        ))
    }
    pub fn who (&mut self, name: &str, o: bool) -> io::IoResult<()> {
        let mut params = vec![name.to_string()];
        if o {
            params.push("o".to_string());
        }
        self.write(Message::new(None, Who, params))
    }
    pub fn whois (&mut self, server: Option<&str>, nickmasks: &[&str]) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Whois,
            vec![].append(server.map(|s| s.to_string()).as_slice())
                  .append([nickmasks.connect(",")])
        ))
    }
    pub fn whowas (&mut self, nickname: &str, count: Option<u32>, server: Option<&str>) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Whowas,
            vec![
                nickname.to_string(),
            ].append(count.map(|s| s.to_string()).as_slice())
             .append(server.map(|s| s.to_string()).as_slice())
        ))
    }

    pub fn kill (&mut self, nickname: &str, comment: &str) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Kill,
            vec![nickname.to_string(), comment.to_string()]
        ))
    }
    pub fn pong (&mut self, daemon1: &str) -> io::IoResult<()> {
        self.write(Message::new(None, Pong, vec![daemon1.to_string()]))
    }

    pub fn away (&mut self, message: Option<&str>) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Away,
            vec![].append(message.map(|s| s.to_string()).as_slice())
        ))
    }
    pub fn rehash (&mut self) -> io::IoResult<()> {
        self.write(Message::new(None, Rehash, vec![]))
    }
    pub fn restart (&mut self) -> io::IoResult<()> {
        self.write(Message::new(None, Restart, vec![]))
    }
    pub fn summon (&mut self, user: &str, server: Option<&str>) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Summon,
            vec![
                user.to_string()
            ].append(server.map(|s| s.to_string()).as_slice())
        ))
    }
    pub fn users (&mut self,  server: Option<&str>) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Users,
            vec![].append(server.map(|s| s.to_string()).as_slice())
        ))
    }
    pub fn wallops (&mut self, text: &str) -> io::IoResult<()> {
        self.write(Message::new(None, Summon, vec![text.to_string()]))
    }
    pub fn userhost (&mut self, nicknames: &[&str]) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Userhost,
            nicknames.iter().map(|s| s.to_string()).collect()
        ))
    }
    pub fn ison (&mut self, nicknames: &[&str]) -> io::IoResult<()> {
        self.write(Message::new(
            None,
            Ison,
            nicknames.iter().map(|s| s.to_string()).collect()
        ))
    }
}

pub trait ClientCallbacks {
    // XXX once storing closures in structs works, we'll also want to provide
    // a default CallbackClient impl of Client to allow users to not have to
    // worry about the struct layout for simple cases.
    fn run_loop (mut self, client: &mut Client) -> io::IoError {
        match self.on_client_connect(client) {
            Err(e) => return e,
            _ => { },
        }

        let err = client.run_loop(|client, m| {
            try!(self.on_any_message(client, m));

            if m.is_reply() {
                try!(self.on_reply(client, m));
            }
            else {
                try!(self.on_command(client, m));
            }

            let from = m.from().as_ref().map(|s| s.as_slice());
            let p = m.params().as_slice();
            match *m.message_type() {
                Pass => {
                    match (p.get(0),) {
                        (Some(ref pass),) => {
                            self.on_pass(
                                client, from,
                                pass.as_slice()
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Nick => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref nick), Some(ref hopcount)) => {
                            match from_str(hopcount.as_slice()) {
                                Some(i) => {
                                    self.on_nick(
                                        client, from,
                                        nick.as_slice(), Some(i)
                                    )
                                },
                                _ => self.on_invalid_message(client, m),
                            }
                        },
                        (Some(ref nick), None) => {
                            self.on_nick(
                                client, from,
                                nick.as_slice(), None
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                User => {
                    match (p.get(0), p.get(1), p.get(2), p.get(3)) {
                        (Some(user), Some(host), Some(server), Some(real)) => {
                            self.on_user(
                                client, from,
                                user.as_slice(), host.as_slice(),
                                server.as_slice(), real.as_slice()
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Server => {
                    match (p.get(0), p.get(1), p.get(2)) {
                        (Some(ref name), Some(ref hopcount), Some(ref info)) => {
                            match from_str(hopcount.as_slice()) {
                                Some(i) => {
                                    self.on_server(
                                        client, from,
                                        name.as_slice(), i, info.as_slice()
                                    )
                                },
                                _ => self.on_invalid_message(client, m),
                            }
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Oper => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref user), Some(ref pass)) => {
                            self.on_oper(
                                client, from,
                                user.as_slice(), pass.as_slice()
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Quit => {
                    self.on_quit(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Squit => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref server), Some(ref comment)) => {
                            self.on_squit(
                                client, from,
                                server.as_slice(), comment.as_slice()
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Join => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref channels), Some(ref keys)) => {
                            let channels: Vec<&str> = channels.as_slice().split(',').collect();
                            let keys: Vec<&str> = keys.as_slice().split(',').collect();
                            self.on_join(
                                client, from,
                                channels.as_slice(),
                                keys.as_slice()
                            )
                        },
                        (Some(ref channels), None) => {
                            let channels: Vec<&str> = channels.as_slice().split(',').collect();
                            self.on_join(
                                client, from,
                                channels.as_slice(),
                                []
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Part => {
                    match (p.get(0),) {
                        (Some(ref channels),) => {
                            let channels: Vec<&str> = channels.as_slice().split(',').collect();
                            self.on_part(
                                client, from,
                                channels.as_slice()
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Mode => {
                    match (p.get(0), p.get(1)) {
                        (Some(name), Some(modes))
                            if is_channel(name.as_slice()) => {
                            let params: Vec<&str> = p.slice_from(2).iter().map(|s| s.as_slice()).collect();
                            self.on_channel_mode(
                                client, from,
                                name.as_slice(), modes.as_slice(),
                                params.as_slice()
                            )
                        },
                        (Some(name), Some(modes)) => {
                            self.on_user_mode(
                                client, from,
                                name.as_slice(), modes.as_slice()
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Topic => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref channel), Some(ref topic)) => {
                            self.on_topic(
                                client, from,
                                channel.as_slice(), Some(topic.as_slice())
                            )
                        },
                        (Some(ref channel), None) => {
                            self.on_topic(
                                client, from,
                                channel.as_slice(), None
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Names => {
                    match (p.get(0),) {
                        (Some(ref channels),) => {
                            let channels: Vec<&str> = channels.as_slice().split(',').collect();
                            self.on_names(
                                client, from,
                                channels.as_slice()
                            )
                        },
                        _ => {
                            self.on_names(
                                client, from,
                                []
                            )
                        },
                    }
                },
                List => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref channels), Some(ref server)) => {
                            let channels: Vec<&str> = channels.as_slice().split(',').collect();
                            self.on_list(
                                client, from,
                                channels.as_slice(),
                                Some(server.as_slice())
                            )
                        },
                        (Some(ref channels), None) => {
                            let channels: Vec<&str> = channels.as_slice().split(',').collect();
                            self.on_list(
                                client, from,
                                channels.as_slice(),
                                None
                            )
                        },
                        _ => {
                            self.on_list(
                                client, from,
                                [], None
                            )
                        },
                    }
                },
                Invite => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref nickname), Some(ref channel)) => {
                            self.on_invite(
                                client, from,
                                nickname.as_slice(), channel.as_slice()
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Kick => {
                    match (p.get(0), p.get(1), p.get(2)) {
                        (Some(ref channel), Some(ref user), Some(ref comment)) => {
                            self.on_kick(
                                client, from,
                                channel.as_slice(), user.as_slice(),
                                Some(comment.as_slice())
                            )
                        },
                        (Some(ref channel), Some(ref user), None) => {
                            self.on_kick(
                                client, from,
                                channel.as_slice(), user.as_slice(),
                                None
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Version => {
                    self.on_version(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Stats => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref query), Some(ref server)) => {
                            self.on_stats(
                                client, from,
                                Some(query.as_slice()), Some(server.as_slice())
                            )
                        },
                        (Some(ref query), None) => {
                            self.on_stats(
                                client, from,
                                Some(query.as_slice()), None
                            )
                        },
                        _ => {
                            self.on_stats(
                                client, from,
                                None, None
                            )
                        },
                    }
                },
                Links => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref server), Some(ref mask)) => {
                            self.on_stats(
                                client, from,
                                Some(server.as_slice()), Some(mask.as_slice())
                            )
                        },
                        (Some(ref mask), None) => {
                            self.on_stats(
                                client, from,
                                None, Some(mask.as_slice())
                            )
                        },
                        _ => {
                            self.on_stats(
                                client, from,
                                None, None
                            )
                        },
                    }
                },
                Time => {
                    self.on_time(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Connect => {
                    match (p.get(0), p.get(1), p.get(2)) {
                        (Some(ref server), Some(ref port), Some(ref remote)) => {
                            match from_str(port.as_slice()) {
                                Some(port) => {
                                    self.on_connect(
                                        client, from,
                                        server.as_slice(),
                                        Some(port),
                                        Some(remote.as_slice())
                                    )
                                },
                                _ => self.on_invalid_message(client, m),
                            }
                        },
                        (Some(ref server), Some(ref port), None) => {
                            match from_str(port.as_slice()) {
                                Some(port) => {
                                    self.on_connect(
                                        client, from,
                                        server.as_slice(),
                                        Some(port),
                                        None
                                    )
                                },
                                _ => self.on_invalid_message(client, m),
                            }
                        },
                        (Some(ref server), None, None) => {
                            self.on_connect(
                                client, from,
                                server.as_slice(),
                                None,
                                None
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Trace => {
                    self.on_trace(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Admin => {
                    self.on_admin(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Info => {
                    self.on_info(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Privmsg => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref receivers), Some(ref text)) => {
                            let receivers: Vec<&str> = receivers.as_slice().split(',').collect();
                            self.on_privmsg(
                                client, from,
                                receivers.as_slice(),
                                text.as_slice()
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Notice => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref nickname), Some(ref text)) => {
                            self.on_notice(
                                client, from,
                                nickname.as_slice(),
                                text.as_slice()
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Who => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref name), Some(ref o)) => {
                            match o.as_slice() {
                                "o" => {
                                    self.on_who(
                                        client, from,
                                        name.as_slice(),
                                        true
                                    )
                                },
                                _ => self.on_invalid_message(client, m),
                            }
                        },
                        (Some(ref name), None) => {
                            self.on_who(
                                client, from,
                                name.as_slice(),
                                false
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Whois => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref server), Some(ref nickmasks)) => {
                            let nickmasks: Vec<&str> = nickmasks.as_slice().split(',').collect();
                            self.on_whois(
                                client, from,
                                Some(server.as_slice()),
                                nickmasks.as_slice()
                            )
                        },
                        (Some(ref nickmasks), None) => {
                            let nickmasks: Vec<&str> = nickmasks.as_slice().split(',').collect();
                            self.on_whois(
                                client, from,
                                None,
                                nickmasks.as_slice()
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Whowas => {
                    match (p.get(0), p.get(1), p.get(2)) {
                        (Some(ref nickname), Some(count), Some(ref server)) => {
                            match from_str(count.as_slice()) {
                                Some(i) => {
                                    self.on_whowas(
                                        client, from,
                                        nickname.as_slice(),
                                        Some(i),
                                        Some(server.as_slice()),
                                    )
                                },
                                _ => self.on_invalid_message(client, m),
                            }
                        },
                        (Some(ref nickname), Some(count), None) => {
                            match from_str(count.as_slice()) {
                                Some(i) => {
                                    self.on_whowas(
                                        client, from,
                                        nickname.as_slice(),
                                        Some(i),
                                        None
                                    )
                                },
                                _ => self.on_invalid_message(client, m),
                            }
                        },
                        (Some(ref nickname), None, None) => {
                            self.on_whowas(
                                client, from,
                                nickname.as_slice(),
                                None,
                                None
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Kill => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref nickname), Some(ref comment)) => {
                            self.on_kill(
                                client, from,
                                nickname.as_slice(),
                                comment.as_slice()
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Ping => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref server1), Some(ref server2)) => {
                            self.on_ping(
                                client, from,
                                server1.as_slice(),
                                Some(server2.as_slice())
                            )
                        },
                        (Some(ref server1), None) => {
                            self.on_ping(
                                client, from,
                                server1.as_slice(),
                                None
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Pong => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref daemon1), Some(ref daemon2)) => {
                            self.on_pong(
                                client, from,
                                daemon1.as_slice(),
                                Some(daemon2.as_slice())
                            )
                        },
                        (Some(ref daemon1), None) => {
                            self.on_ping(
                                client, from,
                                daemon1.as_slice(),
                                None
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Error => {
                    match (p.get(0),) {
                        (Some(ref message),) => {
                            self.on_error(
                                client, from,
                                message.as_slice()
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Away => {
                    self.on_away(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Rehash => {
                    self.on_rehash(
                        client, from
                    )
                },
                Restart => {
                    self.on_restart(
                        client, from
                    )
                },
                Summon => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref user), Some(ref server)) => {
                            self.on_summon(
                                client, from,
                                user.as_slice(),
                                Some(server.as_slice())
                            )
                        },
                        (Some(ref user), None) => {
                            self.on_summon(
                                client, from,
                                user.as_slice(),
                                None
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Users => {
                    self.on_users(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Wallops => {
                    match (p.get(0),) {
                        (Some(ref text),) => {
                            self.on_wallops(
                                client, from,
                                text.as_slice()
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Userhost => {
                    match (p.get(0),) {
                        (Some(_),) => {
                            self.on_userhost(
                                client, from,
                                m.params().iter().map(|s| s.as_slice()).collect::<Vec<&str>>().as_slice()
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                Ison => {
                    match (p.get(0),) {
                        (Some(_),) => {
                            self.on_userhost(
                                client, from,
                                m.params().iter().map(|s| s.as_slice()).collect::<Vec<&str>>().as_slice()
                            )
                        },
                        _ => self.on_invalid_message(client, m),
                    }
                },
                RawCommand(_) => {
                    self.on_unknown_command(client, m)
                },
                Reply(i) => {
                    match i {
                        RPL_WELCOME => self.on_rpl_welcome(client, m),
                        RPL_YOURHOST => self.on_rpl_yourhost(client, m),
                        RPL_CREATED => self.on_rpl_created(client, m),
                        RPL_MYINFO => self.on_rpl_myinfo(client, m),
                        RPL_BOUNCE => self.on_rpl_bounce(client, m),
                        RPL_USERHOST => self.on_rpl_userhost(client, m),
                        RPL_ISON => self.on_rpl_ison(client, m),
                        RPL_AWAY => self.on_rpl_away(client, m),
                        RPL_UNAWAY => self.on_rpl_unaway(client, m),
                        RPL_NOWAWAY => self.on_rpl_noaway(client, m),
                        RPL_WHOISUSER => self.on_rpl_whoisuser(client, m),
                        RPL_WHOISSERVER => self.on_rpl_whoisserver(client, m),
                        RPL_WHOISOPERATOR => self.on_rpl_whoisoperator(client, m),
                        RPL_WHOISIDLE => self.on_rpl_whoisidle(client, m),
                        RPL_ENDOFWHOIS => self.on_rpl_endofwhois(client, m),
                        RPL_WHOISCHANNELS => self.on_rpl_whoischannels(client, m),
                        RPL_WHOWASUSER => self.on_rpl_whowasuser(client, m),
                        RPL_ENDOFWHOWAS => self.on_rpl_endofwhowas(client, m),
                        RPL_LISTSTART => self.on_rpl_liststart(client, m),
                        RPL_LIST => self.on_rpl_list(client, m),
                        RPL_LISTEND => self.on_rpl_listend(client, m),
                        RPL_UNIQOPIS => self.on_rpl_uniqopis(client, m),
                        RPL_CHANNELMODEIS => self.on_rpl_channelmodeis(client, m),
                        RPL_NOTOPIC => self.on_rpl_notopic(client, m),
                        RPL_TOPIC => self.on_rpl_topic(client, m),
                        RPL_INVITING => self.on_rpl_inviting(client, m),
                        RPL_SUMMONING => self.on_rpl_summoning(client, m),
                        RPL_INVITELIST => self.on_rpl_invitelist(client, m),
                        RPL_ENDOFINVITELIST => self.on_rpl_endofinvitelist(client, m),
                        RPL_EXCEPTLIST => self.on_rpl_exceptlist(client, m),
                        RPL_ENDOFEXCEPTLIST => self.on_rpl_endofexceptlist(client, m),
                        RPL_VERSION => self.on_rpl_version(client, m),
                        RPL_WHOREPLY => self.on_rpl_whoreply(client, m),
                        RPL_ENDOFWHO => self.on_rpl_endofwho(client, m),
                        RPL_NAMREPLY => self.on_rpl_namreply(client, m),
                        RPL_ENDOFNAMES => self.on_rpl_endofnames(client, m),
                        RPL_LINKS => self.on_rpl_links(client, m),
                        RPL_ENDOFLINKS => self.on_rpl_endoflinks(client, m),
                        RPL_BANLIST => self.on_rpl_banlist(client, m),
                        RPL_ENDOFBANLIST => self.on_rpl_endofbanlist(client, m),
                        RPL_INFO => self.on_rpl_info(client, m),
                        RPL_ENDOFINFO => self.on_rpl_endofinfo(client, m),
                        RPL_MOTDSTART => self.on_rpl_motdstart(client, m),
                        RPL_MOTD => self.on_rpl_motd(client, m),
                        RPL_ENDOFMOTD => self.on_rpl_endofmotd(client, m),
                        RPL_YOUREOPER => self.on_rpl_youreoper(client, m),
                        RPL_REHASHING => self.on_rpl_rehashing(client, m),
                        RPL_YOURESERVICE => self.on_rpl_youreservice(client, m),
                        RPL_TIME => self.on_rpl_time(client, m),
                        RPL_USERSSTART => self.on_rpl_usersstart(client, m),
                        RPL_USERS => self.on_rpl_users(client, m),
                        RPL_ENDOFUSERS => self.on_rpl_endofusers(client, m),
                        RPL_NOUSERS => self.on_rpl_nousers(client, m),
                        RPL_TRACELINK => self.on_rpl_tracelink(client, m),
                        RPL_TRACECONNECTING => self.on_rpl_traceconnecting(client, m),
                        RPL_TRACEHANDSHAKE => self.on_rpl_tracehandshake(client, m),
                        RPL_TRACEUNKNOWN => self.on_rpl_traceunknown(client, m),
                        RPL_TRACEOPERATOR => self.on_rpl_traceoperator(client, m),
                        RPL_TRACEUSER => self.on_rpl_traceuser(client, m),
                        RPL_TRACESERVER => self.on_rpl_traceserver(client, m),
                        RPL_TRACESERVICE => self.on_rpl_traceservice(client, m),
                        RPL_TRACENEWTYPE => self.on_rpl_tracenewtype(client, m),
                        RPL_TRACECLASS => self.on_rpl_traceclass(client, m),
                        RPL_TRACERECONNECT => self.on_rpl_tracereconnect(client, m),
                        RPL_TRACELOG => self.on_rpl_tracelog(client, m),
                        RPL_TRACEEND => self.on_rpl_traceend(client, m),
                        RPL_STATSLINKINFO => self.on_rpl_statslinkinfo(client, m),
                        RPL_STATSCOMMANDS => self.on_rpl_statscommands(client, m),
                        RPL_ENDOFSTATS => self.on_rpl_endofstats(client, m),
                        RPL_STATSUPTIME => self.on_rpl_statsuptime(client, m),
                        RPL_STATSOLINE => self.on_rpl_statsoline(client, m),
                        RPL_UMODEIS => self.on_rpl_umodeis(client, m),
                        RPL_SERVLIST => self.on_rpl_servlist(client, m),
                        RPL_SERVLISTEND => self.on_rpl_servlistend(client, m),
                        RPL_LUSERCLIENT => self.on_rpl_luserclient(client, m),
                        RPL_LUSEROP => self.on_rpl_luserop(client, m),
                        RPL_LUSERUNKNOWN => self.on_rpl_luserunknown(client, m),
                        RPL_LUSERCHANNELS => self.on_rpl_luserchannels(client, m),
                        RPL_LUSERME => self.on_rpl_luserme(client, m),
                        RPL_ADMINME => self.on_rpl_adminme(client, m),
                        RPL_ADMINLOC1 => self.on_rpl_adminloc1(client, m),
                        RPL_ADMINLOC2 => self.on_rpl_adminloc2(client, m),
                        RPL_ADMINEMAIL => self.on_rpl_adminemail(client, m),
                        RPL_TRYAGAIN => self.on_rpl_tryagain(client, m),
                        ERR_NOSUCHNICK => self.on_err_nosuchnick(client, m),
                        ERR_NOSUCHSERVER => self.on_err_nosuchserver(client, m),
                        ERR_NOSUCHCHANNEL => self.on_err_nosuchchannel(client, m),
                        ERR_CANNOTSENDTOCHAN => self.on_err_cannotsendtochan(client, m),
                        ERR_TOOMANYCHANNELS => self.on_err_toomanychannels(client, m),
                        ERR_WASNOSUCHNICK => self.on_err_wasnosuchnick(client, m),
                        ERR_TOOMANYTARGETS => self.on_err_toomanytargets(client, m),
                        ERR_NOSUCHSERVICE => self.on_err_nosuchservice(client, m),
                        ERR_NOORIGIN => self.on_err_noorigin(client, m),
                        ERR_NORECIPIENT => self.on_err_norecipient(client, m),
                        ERR_NOTEXTTOSEND => self.on_err_notexttosend(client, m),
                        ERR_NOTOPLEVEL => self.on_err_notoplevel(client, m),
                        ERR_WILDTOPLEVEL => self.on_err_wildtoplevel(client, m),
                        ERR_BADMASK => self.on_err_badmask(client, m),
                        ERR_UNKNOWNCOMMAND => self.on_err_unknowncommand(client, m),
                        ERR_NOMOTD => self.on_err_nomotd(client, m),
                        ERR_NOADMININFO => self.on_err_noadmininfo(client, m),
                        ERR_FILEERROR => self.on_err_fileerror(client, m),
                        ERR_NONICKNAMEGIVEN => self.on_err_nonicknamegiven(client, m),
                        ERR_ERRONEUSNICKNAME => self.on_err_erroneusnickname(client, m),
                        ERR_NICKNAMEINUSE => self.on_err_nicknameinuse(client, m),
                        ERR_NICKCOLLISION => self.on_err_nickcollision(client, m),
                        ERR_UNAVAILRESOURCE => self.on_err_unavailresource(client, m),
                        ERR_USERNOTINCHANNEL => self.on_err_usernotinchannel(client, m),
                        ERR_NOTONCHANNEL => self.on_err_notonchannel(client, m),
                        ERR_USERONCHANNEL => self.on_err_useronchannel(client, m),
                        ERR_NOLOGIN => self.on_err_nologin(client, m),
                        ERR_SUMMONDISABLED => self.on_err_summondisabled(client, m),
                        ERR_USERSDISABLED => self.on_err_usersdisabled(client, m),
                        ERR_NOTREGISTERED => self.on_err_notregistered(client, m),
                        ERR_NEEDMOREPARAMS => self.on_err_needmoreparams(client, m),
                        ERR_ALREADYREGISTERED => self.on_err_alreadyregistered(client, m),
                        ERR_NOPERMFORHOST => self.on_err_nopermforhost(client, m),
                        ERR_PASSWDMISMATCH => self.on_err_passwdmismatch(client, m),
                        ERR_YOUREBANNEDCREEP => self.on_err_yourebannedcreep(client, m),
                        ERR_YOUWILLBEBANNED => self.on_err_youwillbebanned(client, m),
                        ERR_KEYSET => self.on_err_keyset(client, m),
                        ERR_CHANNELISFULL => self.on_err_channelisfull(client, m),
                        ERR_UNKNOWNMODE => self.on_err_unknownmode(client, m),
                        ERR_INVITEONLYCHAN => self.on_err_inviteonlychan(client, m),
                        ERR_BANNEDFROMCHAN => self.on_err_bannedfromchan(client, m),
                        ERR_BADCHANNELKEY => self.on_err_badchannelkey(client, m),
                        ERR_BADCHANMASK => self.on_err_badchanmask(client, m),
                        ERR_NOCHANMODES => self.on_err_nochanmodes(client, m),
                        ERR_BANLISTFULL => self.on_err_banlistfull(client, m),
                        ERR_NOPRIVILEGES => self.on_err_noprivileges(client, m),
                        ERR_CHANOPRIVSNEEDED => self.on_err_chanoprivsneeded(client, m),
                        ERR_CANTKILLSERVER => self.on_err_cantkillserver(client, m),
                        ERR_RESTRICTED => self.on_err_restricted(client, m),
                        ERR_UNIQOPPRIVSNEEDED => self.on_err_uniqopprivsneeded(client, m),
                        ERR_NOOPERHOST => self.on_err_nooperhost(client, m),
                        ERR_UMODEUNKNOWNFLAG => self.on_err_umodeunknownflag(client, m),
                        ERR_USERSDONTMATCH => self.on_err_usersdontmatch(client, m),
                        RPL_SERVICEINFO => self.on_rpl_serviceinfo(client, m),
                        RPL_ENDOFSERVICES => self.on_rpl_endofservices(client, m),
                        RPL_SERVICE => self.on_rpl_service(client, m),
                        RPL_NONE => self.on_rpl_none(client, m),
                        RPL_WHOISCHANOP => self.on_rpl_whoischanop(client, m),
                        RPL_KILLDONE => self.on_rpl_killdone(client, m),
                        RPL_CLOSING => self.on_rpl_closing(client, m),
                        RPL_CLOSEEND => self.on_rpl_closeend(client, m),
                        RPL_INFOSTART => self.on_rpl_infostart(client, m),
                        RPL_MYPORTIS => self.on_rpl_myportis(client, m),
                        RPL_STATSCLINE => self.on_rpl_statscline(client, m),
                        RPL_STATSNLINE => self.on_rpl_statsnline(client, m),
                        RPL_STATSILINE => self.on_rpl_statsiline(client, m),
                        RPL_STATSKLINE => self.on_rpl_statskline(client, m),
                        RPL_STATSQLINE => self.on_rpl_statsqline(client, m),
                        RPL_STATSYLINE => self.on_rpl_statsyline(client, m),
                        RPL_STATSVLINE => self.on_rpl_statsvline(client, m),
                        RPL_STATSLLINE => self.on_rpl_statslline(client, m),
                        RPL_STATSHLINE => self.on_rpl_statshline(client, m),
                        RPL_STATSPING => self.on_rpl_statsping(client, m),
                        RPL_STATSBLINE => self.on_rpl_statsbline(client, m),
                        RPL_STATSDLINE => self.on_rpl_statsdline(client, m),
                        ERR_NOSERVICEHOST => self.on_err_noservicehost(client, m),
                        RPL_TOPICDATE => self.on_rpl_topicdate(client, m),
                        ERR_MSGFORBIDDEN => self.on_err_msgforbidden(client, m),
                        _ => self.on_unknown_reply(client, m),
                    }
                },
            }
        });

        let _ = self.on_client_disconnect(client);

        err
    }

    fn on_client_connect (&mut self, client: &mut Client) -> io::IoResult<()> {
        let nick = client.builder().nick.clone();
        let pass = client.builder().pass.clone();
        let username = client.builder().username.clone();
        let servername = client.builder().servername.clone();
        let realname = client.builder().realname.clone();

        match pass {
            Some(pass) => try!(client.pass(pass.as_slice())),
            None => {},
        }

        try!(client.nick(nick.as_slice()));

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

        try!(client.user(
            username.as_slice(),
            hostname.as_slice(),
            servername.as_slice(),
            realname.as_slice(),
        ));

        Ok(())
    }
    #[allow(unused_variable)] fn on_client_disconnect (&mut self, client: &mut Client) -> io::IoResult<()> { Ok(()) }

    #[allow(unused_variable)] fn on_any_message (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_invalid_message (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }

    #[allow(unused_variable)] fn on_unknown_command (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_unknown_reply (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }

    #[allow(unused_variable)] fn on_command (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_reply (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }

    #[allow(unused_variable)] fn on_pass (&mut self, client: &mut Client, from: Option<&str>, pass: &str) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_nick (&mut self, client: &mut Client, from: Option<&str>, nick: &str, hopcount: Option<u32>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_user (&mut self, client: &mut Client, from: Option<&str>, username: &str, hostname: &str, servername: &str, realname: &str) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_server (&mut self, client: &mut Client, from: Option<&str>, servername: &str, hopcount: u32, info: &str) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_oper (&mut self, client: &mut Client, from: Option<&str>, user: &str, pass: &str) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_quit (&mut self, client: &mut Client, from: Option<&str>, msg: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_squit (&mut self, client: &mut Client, from: Option<&str>, server: &str, comment: &str) -> io::IoResult<()> { Ok(()) }

    #[allow(unused_variable)] fn on_join (&mut self, client: &mut Client, from: Option<&str>, channels: &[&str], keys: &[&str]) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_part (&mut self, client: &mut Client, from: Option<&str>, channels: &[&str]) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_channel_mode (&mut self, client: &mut Client, from: Option<&str>, channel: &str, modes: &str, params: &[&str]) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_user_mode (&mut self, client: &mut Client, from: Option<&str>, nickname: &str, modes: &str) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_topic (&mut self, client: &mut Client, from: Option<&str>, channel: &str, topic: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_names (&mut self, client: &mut Client, from: Option<&str>, channels: &[&str]) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_list (&mut self, client: &mut Client, from: Option<&str>, channels: &[&str], server: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_invite (&mut self, client: &mut Client, from: Option<&str>, nickname: &str, channel: &str) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_kick (&mut self, client: &mut Client, from: Option<&str>, channel: &str, user: &str, comment: Option<&str>) -> io::IoResult<()> { Ok(()) }

    #[allow(unused_variable)] fn on_version (&mut self, client: &mut Client, from: Option<&str>, server: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_stats (&mut self, client: &mut Client, from: Option<&str>, query: Option<&str>, server: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_links (&mut self, client: &mut Client, from: Option<&str>, remote_server: Option<&str>, server_mask: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_time (&mut self, client: &mut Client, from: Option<&str>, server: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_connect (&mut self, client: &mut Client, from: Option<&str>, target_server: &str, port: Option<u16>, remote_server: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_trace (&mut self, client: &mut Client, from: Option<&str>, server: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_admin (&mut self, client: &mut Client, from: Option<&str>, server: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_info (&mut self, client: &mut Client, from: Option<&str>, server: Option<&str>) -> io::IoResult<()> { Ok(()) }

    #[allow(unused_variable)] fn on_privmsg (&mut self, client: &mut Client, from: Option<&str>, receivers: &[&str], text: &str) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_notice (&mut self, client: &mut Client, from: Option<&str>, nickname: &str, text: &str) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_who (&mut self, client: &mut Client, from: Option<&str>, name: &str, o: bool) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_whois (&mut self, client: &mut Client, from: Option<&str>, server: Option<&str>, nickmasks: &[&str]) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_whowas (&mut self, client: &mut Client, from: Option<&str>, nickname: &str, count: Option<u32>, server: Option<&str>) -> io::IoResult<()> { Ok(()) }

    #[allow(unused_variable)] fn on_kill (&mut self, client: &mut Client, from: Option<&str>, nickname: &str, comment: &str) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_ping (&mut self, client: &mut Client, from: Option<&str>, server1: &str, server2: Option<&str>) -> io::IoResult<()> {
        client.pong(server1)
    }
    #[allow(unused_variable)] fn on_pong (&mut self, client: &mut Client, from: Option<&str>, daemon1: &str, daemon2: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_error (&mut self, client: &mut Client, from: Option<&str>, message: &str) -> io::IoResult<()> { Ok(()) }

    #[allow(unused_variable)] fn on_away (&mut self, client: &mut Client, from: Option<&str>, message: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rehash (&mut self, client: &mut Client, from: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_restart (&mut self, client: &mut Client, from: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_summon (&mut self, client: &mut Client, from: Option<&str>, user: &str, server: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_users (&mut self, client: &mut Client, from: Option<&str>, server: Option<&str>) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_wallops (&mut self, client: &mut Client, from: Option<&str>, text: &str) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_userhost (&mut self, client: &mut Client, from: Option<&str>, nicknames: &[&str]) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_ison (&mut self, client: &mut Client, from: Option<&str>, nicknames: &[&str]) -> io::IoResult<()> { Ok(()) }

    #[allow(unused_variable)] fn on_rpl_welcome (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_yourhost (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_created (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_myinfo (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_bounce (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_userhost (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_ison (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_away (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_unaway (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_noaway (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_whoisuser (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_whoisserver (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_whoisoperator (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_whoisidle (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_endofwhois (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_whoischannels (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_whowasuser (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_endofwhowas (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_liststart (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_list (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_listend (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_uniqopis (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_channelmodeis (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_notopic (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_topic (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_inviting (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_summoning (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_invitelist (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_endofinvitelist (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_exceptlist (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_endofexceptlist (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_version (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_whoreply (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_endofwho (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_namreply (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_endofnames (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_links (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_endoflinks (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_banlist (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_endofbanlist (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_info (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_endofinfo (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_motdstart (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_motd (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_endofmotd (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_youreoper (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_rehashing (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_youreservice (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_time (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_usersstart (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_users (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_endofusers (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_nousers (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_tracelink (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_traceconnecting (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_tracehandshake (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_traceunknown (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_traceoperator (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_traceuser (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_traceserver (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_traceservice (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_tracenewtype (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_traceclass (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_tracereconnect (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_tracelog (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_traceend (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statslinkinfo (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statscommands (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_endofstats (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statsuptime (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statsoline (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_umodeis (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_servlist (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_servlistend (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_luserclient (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_luserop (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_luserunknown (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_luserchannels (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_luserme (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_adminme (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_adminloc1 (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_adminloc2 (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_adminemail (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_tryagain (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_nosuchnick (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_nosuchserver (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_nosuchchannel (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_cannotsendtochan (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_toomanychannels (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_wasnosuchnick (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_toomanytargets (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_nosuchservice (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_noorigin (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_norecipient (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_notexttosend (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_notoplevel (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_wildtoplevel (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_badmask (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_unknowncommand (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_nomotd (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_noadmininfo (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_fileerror (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_nonicknamegiven (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_erroneusnickname (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_nicknameinuse (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_nickcollision (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_unavailresource (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_usernotinchannel (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_notonchannel (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_useronchannel (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_nologin (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_summondisabled (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_usersdisabled (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_notregistered (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_needmoreparams (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_alreadyregistered (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_nopermforhost (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_passwdmismatch (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_yourebannedcreep (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_youwillbebanned (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_keyset (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_channelisfull (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_unknownmode (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_inviteonlychan (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_bannedfromchan (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_badchannelkey (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_badchanmask (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_nochanmodes (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_banlistfull (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_noprivileges (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_chanoprivsneeded (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_cantkillserver (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_restricted (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_uniqopprivsneeded (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_nooperhost (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_umodeunknownflag (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_usersdontmatch (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_serviceinfo (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_endofservices (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_service (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_none (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_whoischanop (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_killdone (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_closing (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_closeend (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_infostart (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_myportis (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statscline (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statsnline (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statsiline (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statskline (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statsqline (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statsyline (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statsvline (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statslline (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statshline (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statsping (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statsbline (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_statsdline (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_noservicehost (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_rpl_topicdate (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
    #[allow(unused_variable)] fn on_err_msgforbidden (&mut self, client: &mut Client, m: &Message) -> io::IoResult<()> { Ok(()) }
}

fn is_channel (name: &str) -> bool {
    name.starts_with("#") || name.starts_with("&")
}
