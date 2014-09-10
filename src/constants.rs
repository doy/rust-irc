#![allow(dead_code)]

use std::fmt::{FormatError, Formatter, Show};
use std::from_str::FromStr;

#[deriving(PartialEq, Eq)]
pub enum MessageType {
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
    RawCommand(String),
    Reply(u16),
}

impl Show for MessageType {
    fn fmt (&self, f: &mut Formatter) -> Result<(), FormatError> {
        match self {
            &Pass => try!(write!(f, "PASS")),
            &Nick => try!(write!(f, "NICK")),
            &User => try!(write!(f, "USER")),
            &Server => try!(write!(f, "SERVER")),
            &Oper => try!(write!(f, "OPER")),
            &Quit => try!(write!(f, "QUIT")),
            &Squit => try!(write!(f, "SQUIT")),
            &Join => try!(write!(f, "JOIN")),
            &Part => try!(write!(f, "PART")),
            &Mode => try!(write!(f, "MODE")),
            &Topic => try!(write!(f, "TOPIC")),
            &Names => try!(write!(f, "NAMES")),
            &List => try!(write!(f, "LIST")),
            &Invite => try!(write!(f, "INVITE")),
            &Kick => try!(write!(f, "KICK")),
            &Version => try!(write!(f, "VERSION")),
            &Stats => try!(write!(f, "STATS")),
            &Links => try!(write!(f, "LINKS")),
            &Time => try!(write!(f, "TIME")),
            &Connect => try!(write!(f, "CONNECT")),
            &Trace => try!(write!(f, "TRACE")),
            &Admin => try!(write!(f, "ADMIN")),
            &Info => try!(write!(f, "INFO")),
            &Privmsg => try!(write!(f, "PRIVMSG")),
            &Notice => try!(write!(f, "NOTICE")),
            &Who => try!(write!(f, "WHO")),
            &Whois => try!(write!(f, "WHOIS")),
            &Whowas => try!(write!(f, "WHOWAS")),
            &Kill => try!(write!(f, "KILL")),
            &Ping => try!(write!(f, "PING")),
            &Pong => try!(write!(f, "PONG")),
            &Error => try!(write!(f, "ERROR")),
            &Away => try!(write!(f, "AWAY")),
            &Rehash => try!(write!(f, "REHASH")),
            &Restart => try!(write!(f, "RESTART")),
            &Summon => try!(write!(f, "SUMMON")),
            &Users => try!(write!(f, "USERS")),
            &Wallops => try!(write!(f, "WALLOPS")),
            &Userhost => try!(write!(f, "USERHOST")),
            &Ison => try!(write!(f, "ISON")),
            &RawCommand(ref s) => try!(write!(f, "{}", s)),
            &Reply(i) => try!(write!(f, "{:03}", i)),
        }

        Ok(())
    }
}

impl FromStr for MessageType {
    fn from_str (s: &str) -> Option<MessageType> {
        match s {
            "PASS" => Some(Pass),
            "NICK" => Some(Nick),
            "USER" => Some(User),
            "SERVER" => Some(Server),
            "OPER" => Some(Oper),
            "QUIT" => Some(Quit),
            "SQUIT" => Some(Squit),
            "JOIN" => Some(Join),
            "PART" => Some(Part),
            "MODE" => Some(Mode),
            "TOPIC" => Some(Topic),
            "NAMES" => Some(Names),
            "LIST" => Some(List),
            "INVITE" => Some(Invite),
            "KICK" => Some(Kick),
            "VERSION" => Some(Version),
            "STATS" => Some(Stats),
            "LINKS" => Some(Links),
            "TIME" => Some(Time),
            "CONNECT" => Some(Connect),
            "TRACE" => Some(Trace),
            "ADMIN" => Some(Admin),
            "INFO" => Some(Info),
            "PRIVMSG" => Some(Privmsg),
            "NOTICE" => Some(Notice),
            "WHO" => Some(Who),
            "WHOIS" => Some(Whois),
            "WHOWAS" => Some(Whowas),
            "KILL" => Some(Kill),
            "PING" => Some(Ping),
            "PONG" => Some(Pong),
            "ERROR" => Some(Error),
            "AWAY" => Some(Away),
            "REHASH" => Some(Rehash),
            "RESTART" => Some(Restart),
            "SUMMON" => Some(Summon),
            "USERS" => Some(Users),
            "WALLOPS" => Some(Wallops),
            "USERHOST" => Some(Userhost),
            "ISON" => Some(Ison),
            s => {
                match s.char_at(0) {
                    '0'..'9' => {
                        match from_str(s) {
                            Some(i) => Some(Reply(i)),
                            None => Some(RawCommand(s.to_string())),
                        }
                    },
                    _ => Some(RawCommand(s.to_string())),
                }
            },
        }
    }
}

impl MessageType {
    pub fn is_reply (&self) -> bool {
        match self {
            &Reply(_) => true,
            _ => false,
        }
    }
}

// normal replies
pub static RPL_WELCOME: u16 = 1;
pub static RPL_YOURHOST: u16 = 2;
pub static RPL_CREATED: u16 = 3;
pub static RPL_MYINFO: u16 = 4;
pub static RPL_BOUNCE: u16 = 5;
pub static RPL_USERHOST: u16 = 302;
pub static RPL_ISON: u16 = 303;
pub static RPL_AWAY: u16 = 301;
pub static RPL_UNAWAY: u16 = 305;
pub static RPL_NOWAWAY: u16 = 306;
pub static RPL_WHOISUSER: u16 = 311;
pub static RPL_WHOISSERVER: u16 = 312;
pub static RPL_WHOISOPERATOR: u16 = 313;
pub static RPL_WHOISIDLE: u16 = 317;
pub static RPL_ENDOFWHOIS: u16 = 318;
pub static RPL_WHOISCHANNELS: u16 = 319;
pub static RPL_WHOWASUSER: u16 = 314;
pub static RPL_ENDOFWHOWAS: u16 = 369;
pub static RPL_LISTSTART: u16 = 321;
pub static RPL_LIST: u16 = 322;
pub static RPL_LISTEND: u16 = 323;
pub static RPL_UNIQOPIS: u16 = 325;
pub static RPL_CHANNELMODEIS: u16 = 324;
pub static RPL_NOTOPIC: u16 = 331;
pub static RPL_TOPIC: u16 = 332;
pub static RPL_INVITING: u16 = 341;
pub static RPL_SUMMONING: u16 = 342;
pub static RPL_INVITELIST: u16 = 346;
pub static RPL_ENDOFINVITELIST: u16 = 347;
pub static RPL_EXCEPTLIST: u16 = 348;
pub static RPL_ENDOFEXCEPTLIST: u16 = 349;
pub static RPL_VERSION: u16 = 351;
pub static RPL_WHOREPLY: u16 = 352;
pub static RPL_ENDOFWHO: u16 = 315;
pub static RPL_NAMREPLY: u16 = 353;
pub static RPL_ENDOFNAMES: u16 = 366;
pub static RPL_LINKS: u16 = 364;
pub static RPL_ENDOFLINKS: u16 = 365;
pub static RPL_BANLIST: u16 = 367;
pub static RPL_ENDOFBANLIST: u16 = 368;
pub static RPL_INFO: u16 = 371;
pub static RPL_ENDOFINFO: u16 = 374;
pub static RPL_MOTDSTART: u16 = 375;
pub static RPL_MOTD: u16 = 372;
pub static RPL_ENDOFMOTD: u16 = 376;
pub static RPL_YOUREOPER: u16 = 381;
pub static RPL_REHASHING: u16 = 382;
pub static RPL_YOURESERVICE: u16 = 383;
pub static RPL_TIME: u16 = 391;
pub static RPL_USERSSTART: u16 = 392;
pub static RPL_USERS: u16 = 393;
pub static RPL_ENDOFUSERS: u16 = 394;
pub static RPL_NOUSERS: u16 = 395;
pub static RPL_TRACELINK: u16 = 200;
pub static RPL_TRACECONNECTING: u16 = 201;
pub static RPL_TRACEHANDSHAKE: u16 = 202;
pub static RPL_TRACEUNKNOWN: u16 = 203;
pub static RPL_TRACEOPERATOR: u16 = 204;
pub static RPL_TRACEUSER: u16 = 205;
pub static RPL_TRACESERVER: u16 = 206;
pub static RPL_TRACESERVICE: u16 = 207;
pub static RPL_TRACENEWTYPE: u16 = 208;
pub static RPL_TRACECLASS: u16 = 209;
pub static RPL_TRACERECONNECT: u16 = 210;
pub static RPL_TRACELOG: u16 = 261;
pub static RPL_TRACEEND: u16 = 262;
pub static RPL_STATSLINKINFO: u16 = 211;
pub static RPL_STATSCOMMANDS: u16 = 212;
pub static RPL_ENDOFSTATS: u16 = 219;
pub static RPL_STATSUPTIME: u16 = 242;
pub static RPL_STATSOLINE: u16 = 243;
pub static RPL_UMODEIS: u16 = 221;
pub static RPL_SERVLIST: u16 = 234;
pub static RPL_SERVLISTEND: u16 = 235;
pub static RPL_LUSERCLIENT: u16 = 251;
pub static RPL_LUSEROP: u16 = 252;
pub static RPL_LUSERUNKNOWN: u16 = 253;
pub static RPL_LUSERCHANNELS: u16 = 254;
pub static RPL_LUSERME: u16 = 255;
pub static RPL_ADMINME: u16 = 256;
pub static RPL_ADMINLOC1: u16 = 257;
pub static RPL_ADMINLOC2: u16 = 258;
pub static RPL_ADMINEMAIL: u16 = 259;
pub static RPL_TRYAGAIN: u16 = 263;

// errors
pub static ERR_NOSUCHNICK: u16 = 401; // No such nick/channel
pub static ERR_NOSUCHSERVER: u16 = 402; // No such server
pub static ERR_NOSUCHCHANNEL: u16 = 403; // No such channel
pub static ERR_CANNOTSENDTOCHAN: u16 = 404; // Cannot send to channel
pub static ERR_TOOMANYCHANNELS: u16 = 405; // You have joined too many channels
pub static ERR_WASNOSUCHNICK: u16 = 406; // There was no such nickname
pub static ERR_TOOMANYTARGETS: u16 = 407; // Duplicate recipients. No message delivered
pub static ERR_NOSUCHSERVICE: u16 = 408; // No such service
pub static ERR_NOORIGIN: u16 = 409; // No origin specified
pub static ERR_NORECIPIENT: u16 = 411; // No recipient given
pub static ERR_NOTEXTTOSEND: u16 = 412; // No text to send
pub static ERR_NOTOPLEVEL: u16 = 413; // No toplevel domain specified
pub static ERR_WILDTOPLEVEL: u16 = 414; // Wildcard in toplevel domain
pub static ERR_BADMASK: u16 = 415; // Bad server/host mask
pub static ERR_UNKNOWNCOMMAND: u16 = 421; // Unknown command
pub static ERR_NOMOTD: u16 = 422; // MOTD file is missing
pub static ERR_NOADMININFO: u16 = 423; // No administrative info available
pub static ERR_FILEERROR: u16 = 424; // File error
pub static ERR_NONICKNAMEGIVEN: u16 = 431; // No nickname given
pub static ERR_ERRONEUSNICKNAME: u16 = 432; // Erroneus nickname
pub static ERR_NICKNAMEINUSE: u16 = 433; // Nickname is already in use
pub static ERR_NICKCOLLISION: u16 = 436; // Nickname collision KILL
pub static ERR_UNAVAILRESOURCE: u16 = 437; // Nick/channel is temporarily unavailable
pub static ERR_USERNOTINCHANNEL: u16 = 441; // They aren't on that channel
pub static ERR_NOTONCHANNEL: u16 = 442; // You're not on that channel
pub static ERR_USERONCHANNEL: u16 = 443; // User is already on channel
pub static ERR_NOLOGIN: u16 = 444; // User not logged in
pub static ERR_SUMMONDISABLED: u16 = 445; // SUMMON has been disabled
pub static ERR_USERSDISABLED: u16 = 446; // USERS has been disabled
pub static ERR_NOTREGISTERED: u16 = 451; // You have not registered
pub static ERR_NEEDMOREPARAMS: u16 = 461; // Not enough parameters
pub static ERR_ALREADYREGISTERED: u16 = 462; // You may not reregister
pub static ERR_NOPERMFORHOST: u16 = 463; // Your host isn't among the privileged
pub static ERR_PASSWDMISMATCH: u16 = 464; // Password incorrect
pub static ERR_YOUREBANNEDCREEP: u16 = 465; // You are banned from this server
pub static ERR_YOUWILLBEBANNED: u16 = 466;
pub static ERR_KEYSET: u16 = 467; // Channel key already set
pub static ERR_CHANNELISFULL: u16 = 471; // Cannot join channel (+l)
pub static ERR_UNKNOWNMODE: u16 = 472; // Unknown mode char
pub static ERR_INVITEONLYCHAN: u16 = 473; // Cannot join channel (+i)
pub static ERR_BANNEDFROMCHAN: u16 = 474; // Cannot join channel (+b)
pub static ERR_BADCHANNELKEY: u16 = 475; // Cannot join channel (+k)
pub static ERR_BADCHANMASK: u16 = 476; // Bad channel mask
pub static ERR_NOCHANMODES: u16 = 477; // Channel doesn't support modes
pub static ERR_BANLISTFULL: u16 = 478; // Channel list is full
pub static ERR_NOPRIVILEGES: u16 = 481; // Permission denied- You're not an IRC operator
pub static ERR_CHANOPRIVSNEEDED: u16 = 482; // You're not channel operator
pub static ERR_CANTKILLSERVER: u16 = 483; // You can't kill a server!
pub static ERR_RESTRICTED: u16 = 484; // Your connection is restricted!
pub static ERR_UNIQOPPRIVSNEEDED: u16 = 485; // You're not the original channel operator
pub static ERR_NOOPERHOST: u16 = 491; // No O-lines for your host
pub static ERR_UMODEUNKNOWNFLAG: u16 = 501; // Unknown MODE flag
pub static ERR_USERSDONTMATCH: u16 = 502; // Can't change mode for other users

//unused
pub static RPL_SERVICEINFO: u16 = 231;
pub static RPL_ENDOFSERVICES: u16 = 232;
pub static RPL_SERVICE: u16 = 233;
pub static RPL_NONE: u16 = 300;
pub static RPL_WHOISCHANOP: u16 = 316;
pub static RPL_KILLDONE: u16 = 361;
pub static RPL_CLOSING: u16 = 362;
pub static RPL_CLOSEEND: u16 = 363;
pub static RPL_INFOSTART: u16 = 373;
pub static RPL_MYPORTIS: u16 = 384;
pub static RPL_STATSCLINE: u16 = 213;
pub static RPL_STATSNLINE: u16 = 214;
pub static RPL_STATSILINE: u16 = 215;
pub static RPL_STATSKLINE: u16 = 216;
pub static RPL_STATSQLINE: u16 = 217;
pub static RPL_STATSYLINE: u16 = 218;
pub static RPL_STATSVLINE: u16 = 240;
pub static RPL_STATSLLINE: u16 = 241;
pub static RPL_STATSHLINE: u16 = 244;
pub static RPL_STATSPING: u16 = 246;
pub static RPL_STATSBLINE: u16 = 247;
pub static RPL_STATSDLINE: u16 = 250;
pub static ERR_NOSERVICEHOST: u16 = 492;

// guesses
pub static RPL_TOPICDATE: u16 = 333; // date the topic was set, in seconds since the epoch
pub static ERR_MSGFORBIDDEN: u16 = 505; // freenode blocking privmsg from unreged users

pub static MAX_MESSAGE_LENGTH: uint = 512;

#[test]
fn test_message_type () {
    assert!(!from_str::<MessageType>("PASS").unwrap().is_reply());
    assert!(from_str::<MessageType>("001").unwrap().is_reply());
    assert!(!from_str::<MessageType>("NOTACOMMAND").unwrap().is_reply());
}
