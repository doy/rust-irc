use std::{io, str};

use constants::{
    Pass,
    Nick,
    User,
    Server,
    Oper,
    Quit,
    Squit,
    Join,
    Part,
    Mode,
    Topic,
    Names,
    List,
    Invite,
    Kick,
    Version,
    Stats,
    Links,
    Time,
    Connect,
    Trace,
    Admin,
    Info,
    Privmsg,
    Notice,
    Who,
    Whois,
    Whowas,
    Kill,
    Ping,
    Pong,
    Error,
    Away,
    Rehash,
    Restart,
    Summon,
    Users,
    Wallops,
    Userhost,
    Ison,
    RawCommand,
    Reply,
};
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
        let stream = io::TcpStream::connect(self.servername.as_slice(), self.port);
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

    fn run_loop_with (mut self, handler: |&mut Self, &Message|) -> Self {
        loop {
            let m = self.read();
            handler(&mut self, &m);
        }
        self
    }

    // XXX once storing closures in structs works, we'll also want to provide
    // a default CallbackClient impl of Client to allow users to not have to
    // worry about the struct layout for simple cases.
    fn run_loop (mut self) {
        Client::on_client_connect(&mut self);

        let mut client = self.run_loop_with(|client, m| {
            Client::on_any_message(client, m);

            let from = m.from().as_ref().map(|s| s.as_slice());
            let p = m.params().as_slice();
            match *m.message_type() {
                Pass => {
                    match (p.get(0),) {
                        (Some(ref pass),) => {
                            Client::on_pass(
                                client, from,
                                pass.as_slice()
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Nick => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref nick), Some(ref hopcount)) => {
                            match from_str(hopcount.as_slice()) {
                                Some(i) => {
                                    Client::on_nick(
                                        client, from,
                                        nick.as_slice(), Some(i)
                                    )
                                },
                                _ => Client::on_invalid_message(client, m),
                            }
                        },
                        (Some(ref nick), None) => {
                            Client::on_nick(
                                client, from,
                                nick.as_slice(), None
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                User => {
                    match (p.get(0), p.get(1), p.get(2), p.get(3)) {
                        (Some(user), Some(host), Some(server), Some(real)) => {
                            Client::on_user(
                                client, from,
                                user.as_slice(), host.as_slice(),
                                server.as_slice(), real.as_slice()
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Server => {
                    match (p.get(0), p.get(1), p.get(2)) {
                        (Some(ref name), Some(ref hopcount), Some(ref info)) => {
                            match from_str(hopcount.as_slice()) {
                                Some(i) => {
                                    Client::on_server(
                                        client, from,
                                        name.as_slice(), i, info.as_slice()
                                    )
                                },
                                _ => Client::on_invalid_message(client, m),
                            }
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Oper => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref user), Some(ref pass)) => {
                            Client::on_oper(
                                client, from,
                                user.as_slice(), pass.as_slice()
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Quit => {
                    Client::on_quit(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Squit => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref server), Some(ref comment)) => {
                            Client::on_squit(
                                client, from,
                                server.as_slice(), comment.as_slice()
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Join => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref channels), Some(ref keys)) => {
                            Client::on_join(
                                client, from,
                                channels.as_slice().split(',').collect(),
                                keys.as_slice().split(',').collect()
                            )
                        },
                        (Some(ref channels), None) => {
                            Client::on_join(
                                client, from,
                                channels.as_slice().split(',').collect(),
                                vec![]
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Part => {
                    match (p.get(0),) {
                        (Some(ref channels),) => {
                            Client::on_part(
                                client, from,
                                channels.as_slice().split(',').collect()
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Mode => {
                    match (p.get(0), p.get(1)) {
                        (Some(name), Some(modes))
                            if is_channel(name.as_slice()) => {
                            Client::on_channel_mode(
                                client, from,
                                name.as_slice(), modes.as_slice(),
                                p.slice_from(2).iter().map(|s| s.as_slice()).collect()
                            )
                        },
                        (Some(name), Some(modes)) => {
                            Client::on_user_mode(
                                client, from,
                                name.as_slice(), modes.as_slice()
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Topic => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref channel), Some(ref topic)) => {
                            Client::on_topic(
                                client, from,
                                channel.as_slice(), Some(topic.as_slice())
                            )
                        },
                        (Some(ref channel), None) => {
                            Client::on_topic(
                                client, from,
                                channel.as_slice(), None
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Names => {
                    match (p.get(0),) {
                        (Some(ref channels),) => {
                            Client::on_names(
                                client, from,
                                channels.as_slice().split(',').collect()
                            )
                        },
                        _ => {
                            Client::on_names(
                                client, from,
                                vec![]
                            )
                        },
                    }
                },
                List => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref channels), Some(ref server)) => {
                            Client::on_list(
                                client, from,
                                channels.as_slice().split(',').collect(),
                                Some(server.as_slice())
                            )
                        },
                        (Some(ref channels), None) => {
                            Client::on_list(
                                client, from,
                                channels.as_slice().split(',').collect(),
                                None
                            )
                        },
                        _ => {
                            Client::on_list(
                                client, from,
                                vec![], None
                            )
                        },
                    }
                },
                Invite => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref nickname), Some(ref channel)) => {
                            Client::on_invite(
                                client, from,
                                nickname.as_slice(), channel.as_slice()
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Kick => {
                    match (p.get(0), p.get(1), p.get(2)) {
                        (Some(ref channel), Some(ref user), Some(ref comment)) => {
                            Client::on_kick(
                                client, from,
                                channel.as_slice(), user.as_slice(),
                                Some(comment.as_slice())
                            )
                        },
                        (Some(ref channel), Some(ref user), None) => {
                            Client::on_kick(
                                client, from,
                                channel.as_slice(), user.as_slice(),
                                None
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Version => {
                    Client::on_version(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Stats => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref query), Some(ref server)) => {
                            Client::on_stats(
                                client, from,
                                Some(query.as_slice()), Some(server.as_slice())
                            )
                        },
                        (Some(ref query), None) => {
                            Client::on_stats(
                                client, from,
                                Some(query.as_slice()), None
                            )
                        },
                        _ => {
                            Client::on_stats(
                                client, from,
                                None, None
                            )
                        },
                    }
                },
                Links => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref server), Some(ref mask)) => {
                            Client::on_stats(
                                client, from,
                                Some(server.as_slice()), Some(mask.as_slice())
                            )
                        },
                        (Some(ref mask), None) => {
                            Client::on_stats(
                                client, from,
                                None, Some(mask.as_slice())
                            )
                        },
                        _ => {
                            Client::on_stats(
                                client, from,
                                None, None
                            )
                        },
                    }
                },
                Time => {
                    Client::on_time(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Connect => {
                    match (p.get(0), p.get(1), p.get(2)) {
                        (Some(ref server), Some(ref port), Some(ref remote)) => {
                            match from_str(port.as_slice()) {
                                Some(port) => {
                                    Client::on_connect(
                                        client, from,
                                        server.as_slice(),
                                        Some(port),
                                        Some(remote.as_slice())
                                    )
                                },
                                _ => Client::on_invalid_message(client, m),
                            }
                        },
                        (Some(ref server), Some(ref port), None) => {
                            match from_str(port.as_slice()) {
                                Some(port) => {
                                    Client::on_connect(
                                        client, from,
                                        server.as_slice(),
                                        Some(port),
                                        None
                                    )
                                },
                                _ => Client::on_invalid_message(client, m),
                            }
                        },
                        (Some(ref server), None, None) => {
                            Client::on_connect(
                                client, from,
                                server.as_slice(),
                                None,
                                None
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Trace => {
                    Client::on_trace(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Admin => {
                    Client::on_admin(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Info => {
                    Client::on_info(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Privmsg => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref receivers), Some(ref text)) => {
                            Client::on_privmsg(
                                client, from,
                                receivers.as_slice().split(',').collect(),
                                text.as_slice()
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Notice => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref nickname), Some(ref text)) => {
                            Client::on_notice(
                                client, from,
                                nickname.as_slice(),
                                text.as_slice()
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Who => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref name), Some(ref o)) => {
                            match o.as_slice() {
                                "o" => {
                                    Client::on_who(
                                        client, from,
                                        name.as_slice(),
                                        true
                                    )
                                },
                                _ => Client::on_invalid_message(client, m),
                            }
                        },
                        (Some(ref name), None) => {
                            Client::on_who(
                                client, from,
                                name.as_slice(),
                                false
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Whois => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref server), Some(ref nickmasks)) => {
                            Client::on_whois(
                                client, from,
                                Some(server.as_slice()),
                                nickmasks.as_slice().split(',').collect()
                            )
                        },
                        (Some(ref nickmasks), None) => {
                            Client::on_whois(
                                client, from,
                                None,
                                nickmasks.as_slice().split(',').collect()
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Whowas => {
                    match (p.get(0), p.get(1), p.get(2)) {
                        (Some(ref nickname), Some(count), Some(ref server)) => {
                            match from_str(count.as_slice()) {
                                Some(i) => {
                                    Client::on_whowas(
                                        client, from,
                                        nickname.as_slice(),
                                        Some(i),
                                        Some(server.as_slice()),
                                    )
                                },
                                _ => Client::on_invalid_message(client, m),
                            }
                        },
                        (Some(ref nickname), Some(count), None) => {
                            match from_str(count.as_slice()) {
                                Some(i) => {
                                    Client::on_whowas(
                                        client, from,
                                        nickname.as_slice(),
                                        Some(i),
                                        None
                                    )
                                },
                                _ => Client::on_invalid_message(client, m),
                            }
                        },
                        (Some(ref nickname), None, None) => {
                            Client::on_whowas(
                                client, from,
                                nickname.as_slice(),
                                None,
                                None
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Kill => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref nickname), Some(ref comment)) => {
                            Client::on_kill(
                                client, from,
                                nickname.as_slice(),
                                comment.as_slice()
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Ping => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref server1), Some(ref server2)) => {
                            Client::on_ping(
                                client, from,
                                server1.as_slice(),
                                Some(server2.as_slice())
                            )
                        },
                        (Some(ref server1), None) => {
                            Client::on_ping(
                                client, from,
                                server1.as_slice(),
                                None
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Pong => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref daemon1), Some(ref daemon2)) => {
                            Client::on_pong(
                                client, from,
                                daemon1.as_slice(),
                                Some(daemon2.as_slice())
                            )
                        },
                        (Some(ref daemon1), None) => {
                            Client::on_ping(
                                client, from,
                                daemon1.as_slice(),
                                None
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Error => {
                    match (p.get(0),) {
                        (Some(ref message),) => {
                            Client::on_error(
                                client, from,
                                message.as_slice()
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Away => {
                    Client::on_away(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Rehash => {
                    Client::on_rehash(
                        client, from
                    )
                },
                Restart => {
                    Client::on_restart(
                        client, from
                    )
                },
                Summon => {
                    match (p.get(0), p.get(1)) {
                        (Some(ref user), Some(ref server)) => {
                            Client::on_summon(
                                client, from,
                                user.as_slice(),
                                Some(server.as_slice())
                            )
                        },
                        (Some(ref user), None) => {
                            Client::on_summon(
                                client, from,
                                user.as_slice(),
                                None
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Users => {
                    Client::on_users(
                        client, from,
                        p.get(0).map(|s| s.as_slice())
                    )
                },
                Wallops => {
                    match (p.get(0),) {
                        (Some(ref text),) => {
                            Client::on_wallops(
                                client, from,
                                text.as_slice()
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Userhost => {
                    match (p.get(0),) {
                        (Some(_),) => {
                            Client::on_userhost(
                                client, from,
                                m.params().iter().map(|s| s.as_slice()).collect::<Vec<&str>>().as_slice()
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                Ison => {
                    match (p.get(0),) {
                        (Some(_),) => {
                            Client::on_userhost(
                                client, from,
                                m.params().iter().map(|s| s.as_slice()).collect::<Vec<&str>>().as_slice()
                            )
                        },
                        _ => Client::on_invalid_message(client, m),
                    }
                },
                RawCommand(_) => {
                    Client::on_unknown_message(client, m)
                },
                Reply(_) => {
                    // XXX
                },
            }
        });

        Client::on_client_disconnect(&mut client);
    }

    fn on_client_connect (client: &mut Self) {
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
    #[allow(unused_variable)] fn on_client_disconnect (client: &mut Self) { }

    #[allow(unused_variable)] fn on_any_message (client: &mut Self, m: &Message) { }
    #[allow(unused_variable)] fn on_invalid_message (client: &mut Self, m: &Message) { }
    #[allow(unused_variable)] fn on_unknown_message (client: &mut Self, m: &Message) { }

    #[allow(unused_variable)] fn on_pass (client: &mut Self, from: Option<&str>, pass: &str) { }
    #[allow(unused_variable)] fn on_nick (client: &mut Self, from: Option<&str>, nick: &str, hopcount: Option<u32>) { }
    #[allow(unused_variable)] fn on_user (client: &mut Self, from: Option<&str>, username: &str, hostname: &str, servername: &str, realname: &str) { }
    #[allow(unused_variable)] fn on_server (client: &mut Self, from: Option<&str>, servername: &str, hopcount: u32, info: &str) { }
    #[allow(unused_variable)] fn on_oper (client: &mut Self, from: Option<&str>, user: &str, pass: &str) { }
    #[allow(unused_variable)] fn on_quit (client: &mut Self, from: Option<&str>, msg: Option<&str>) { }
    #[allow(unused_variable)] fn on_squit (client: &mut Self, from: Option<&str>, server: &str, comment: &str) { }

    #[allow(unused_variable)] fn on_join (client: &mut Self, from: Option<&str>, channels: Vec<&str>, keys: Vec<&str>) { }
    #[allow(unused_variable)] fn on_part (client: &mut Self, from: Option<&str>, channels: Vec<&str>) { }
    #[allow(unused_variable)] fn on_channel_mode (client: &mut Self, from: Option<&str>, channel: &str, modes: &str, params: Vec<&str>) { }
    #[allow(unused_variable)] fn on_user_mode (client: &mut Self, from: Option<&str>, nickname: &str, modes: &str) { }
    #[allow(unused_variable)] fn on_topic (client: &mut Self, from: Option<&str>, channel: &str, topic: Option<&str>) { }
    #[allow(unused_variable)] fn on_names (client: &mut Self, from: Option<&str>, channels: Vec<&str>) { }
    #[allow(unused_variable)] fn on_list (client: &mut Self, from: Option<&str>, channels: Vec<&str>, server: Option<&str>) { }
    #[allow(unused_variable)] fn on_invite (client: &mut Self, from: Option<&str>, nickname: &str, channel: &str) { }
    #[allow(unused_variable)] fn on_kick (client: &mut Self, from: Option<&str>, channel: &str, user: &str, comment: Option<&str>) { }

    #[allow(unused_variable)] fn on_version (client: &mut Self, from: Option<&str>, server: Option<&str>) { }
    #[allow(unused_variable)] fn on_stats (client: &mut Self, from: Option<&str>, query: Option<&str>, server: Option<&str>) { }
    #[allow(unused_variable)] fn on_links (client: &mut Self, from: Option<&str>, remote_server: Option<&str>, server_mask: Option<&str>) { }
    #[allow(unused_variable)] fn on_time (client: &mut Self, from: Option<&str>, server: Option<&str>) { }
    #[allow(unused_variable)] fn on_connect (client: &mut Self, from: Option<&str>, target_server: &str, port: Option<u16>, remote_server: Option<&str>) { }
    #[allow(unused_variable)] fn on_trace (client: &mut Self, from: Option<&str>, server: Option<&str>) { }
    #[allow(unused_variable)] fn on_admin (client: &mut Self, from: Option<&str>, server: Option<&str>) { }
    #[allow(unused_variable)] fn on_info (client: &mut Self, from: Option<&str>, server: Option<&str>) { }

    #[allow(unused_variable)] fn on_privmsg (client: &mut Self, from: Option<&str>, receivers: Vec<&str>, text: &str) { }
    #[allow(unused_variable)] fn on_notice (client: &mut Self, from: Option<&str>, nickname: &str, text: &str) { }
    #[allow(unused_variable)] fn on_who (client: &mut Self, from: Option<&str>, name: &str, o: bool) { }
    #[allow(unused_variable)] fn on_whois (client: &mut Self, from: Option<&str>, server: Option<&str>, nickmasks: Vec<&str>) { }
    #[allow(unused_variable)] fn on_whowas (client: &mut Self, from: Option<&str>, nickname: &str, count: Option<u32>, server: Option<&str>) { }

    #[allow(unused_variable)] fn on_kill (client: &mut Self, from: Option<&str>, nickname: &str, comment: &str) { }
    #[allow(unused_variable)] fn on_ping (client: &mut Self, from: Option<&str>, server1: &str, server2: Option<&str>) { }
    #[allow(unused_variable)] fn on_pong (client: &mut Self, from: Option<&str>, daemon1: &str, daemon2: Option<&str>) { }
    #[allow(unused_variable)] fn on_error (client: &mut Self, from: Option<&str>, message: &str) { }

    #[allow(unused_variable)] fn on_away (client: &mut Self, from: Option<&str>, message: Option<&str>) { }
    #[allow(unused_variable)] fn on_rehash (client: &mut Self, from: Option<&str>) { }
    #[allow(unused_variable)] fn on_restart (client: &mut Self, from: Option<&str>) { }
    #[allow(unused_variable)] fn on_summon (client: &mut Self, from: Option<&str>, user: &str, server: Option<&str>) { }
    #[allow(unused_variable)] fn on_users (client: &mut Self, from: Option<&str>, server: Option<&str>) { }
    #[allow(unused_variable)] fn on_wallops (client: &mut Self, from: Option<&str>, text: &str) { }
    #[allow(unused_variable)] fn on_userhost (client: &mut Self, from: Option<&str>, nicknames: &[&str]) { }
    #[allow(unused_variable)] fn on_ison (client: &mut Self, from: Option<&str>, nicknames: &[&str]) { }
}

fn is_channel (name: &str) -> bool {
    name.starts_with("#") || name.starts_with("&")
}
