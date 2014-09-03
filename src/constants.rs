use std::from_str::FromStr;

#[deriving(PartialEq, Eq, Show)]
pub enum Command {
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
    Raw(String),
}

impl FromStr for Command {
    fn from_str (s: &str) -> Option<Command> {
        match s {
            s if s == "PASS" => Some(Pass),
            s if s == "NICK" => Some(Nick),
            s if s == "USER" => Some(User),
            s if s == "SERVER" => Some(Server),
            s if s == "OPER" => Some(Oper),
            s if s == "QUIT" => Some(Quit),
            s if s == "SQUIT" => Some(Squit),
            s if s == "JOIN" => Some(Join),
            s if s == "PART" => Some(Part),
            s if s == "MODE" => Some(Mode),
            s if s == "TOPIC" => Some(Topic),
            s if s == "NAMES" => Some(Names),
            s if s == "LIST" => Some(List),
            s if s == "INVITE" => Some(Invite),
            s if s == "KICK" => Some(Kick),
            s if s == "VERSION" => Some(Version),
            s if s == "STATS" => Some(Stats),
            s if s == "LINKS" => Some(Links),
            s if s == "TIME" => Some(Time),
            s if s == "CONNECT" => Some(Connect),
            s if s == "TRACE" => Some(Trace),
            s if s == "ADMIN" => Some(Admin),
            s if s == "INFO" => Some(Info),
            s if s == "PRIVMSG" => Some(Privmsg),
            s if s == "NOTICE" => Some(Notice),
            s if s == "WHO" => Some(Who),
            s if s == "WHOIS" => Some(Whois),
            s if s == "WHOWAS" => Some(Whowas),
            s if s == "KILL" => Some(Kill),
            s if s == "PING" => Some(Ping),
            s if s == "PONG" => Some(Pong),
            s if s == "ERROR" => Some(Error),
            s if s == "AWAY" => Some(Away),
            s if s == "REHASH" => Some(Rehash),
            s if s == "RESTART" => Some(Restart),
            s if s == "SUMMON" => Some(Summon),
            s if s == "USERS" => Some(Users),
            s if s == "WALLOPS" => Some(Wallops),
            s if s == "USERHOST" => Some(Userhost),
            s if s == "ISON" => Some(Ison),
            s => Some(Raw(s.to_string())),
        }
    }
}

#[deriving(PartialEq, Eq, Show)]
pub enum ReplyId {
    // normal replies
    RPL_WELCOME = 1,
    RPL_YOURHOST = 2,
    RPL_CREATED = 3,
    RPL_MYINFO = 4,
    RPL_BOUNCE = 5,
    RPL_USERHOST = 302,
    RPL_ISON = 303,
    RPL_AWAY = 301,
    RPL_UNAWAY = 305,
    RPL_NOWAWAY = 306,
    RPL_WHOISUSER = 311,
    RPL_WHOISSERVER = 312,
    RPL_WHOISOPERATOR = 313,
    RPL_WHOISIDLE = 317,
    RPL_ENDOFWHOIS = 318,
    RPL_WHOISCHANNELS = 319,
    RPL_WHOWASUSER = 314,
    RPL_ENDOFWHOWAS = 369,
    RPL_LISTSTART = 321,
    RPL_LIST = 322,
    RPL_LISTEND = 323,
    RPL_UNIQOPIS = 325,
    RPL_CHANNELMODEIS = 324,
    RPL_NOTOPIC = 331,
    RPL_TOPIC = 332,
    RPL_INVITING = 341,
    RPL_SUMMONING = 342,
    RPL_INVITELIST = 346,
    RPL_ENDOFINVITELIST = 347,
    RPL_EXCEPTLIST = 348,
    RPL_ENDOFEXCEPTLIST = 349,
    RPL_VERSION = 351,
    RPL_WHOREPLY = 352,
    RPL_ENDOFWHO = 315,
    RPL_NAMREPLY = 353,
    RPL_ENDOFNAMES = 366,
    RPL_LINKS = 364,
    RPL_ENDOFLINKS = 365,
    RPL_BANLIST = 367,
    RPL_ENDOFBANLIST = 368,
    RPL_INFO = 371,
    RPL_ENDOFINFO = 374,
    RPL_MOTDSTART = 375,
    RPL_MOTD = 372,
    RPL_ENDOFMOTD = 376,
    RPL_YOUREOPER = 381,
    RPL_REHASHING = 382,
    RPL_YOURESERVICE = 383,
    RPL_TIME = 391,
    RPL_USERSSTART = 392,
    RPL_USERS = 393,
    RPL_ENDOFUSERS = 394,
    RPL_NOUSERS = 395,
    RPL_TRACELINK = 200,
    RPL_TRACECONNECTING = 201,
    RPL_TRACEHANDSHAKE = 202,
    RPL_TRACEUNKNOWN = 203,
    RPL_TRACEOPERATOR = 204,
    RPL_TRACEUSER = 205,
    RPL_TRACESERVER = 206,
    RPL_TRACESERVICE = 207,
    RPL_TRACENEWTYPE = 208,
    RPL_TRACECLASS = 209,
    RPL_TRACERECONNECT = 210,
    RPL_TRACELOG = 261,
    RPL_TRACEEND = 262,
    RPL_STATSLINKINFO = 211,
    RPL_STATSCOMMANDS = 212,
    RPL_ENDOFSTATS = 219,
    RPL_STATSUPTIME = 242,
    RPL_STATSOLINE = 243,
    RPL_UMODEIS = 221,
    RPL_SERVLIST = 234,
    RPL_SERVLISTEND = 235,
    RPL_LUSERCLIENT = 251,
    RPL_LUSEROP = 252,
    RPL_LUSERUNKNOWN = 253,
    RPL_LUSERCHANNELS = 254,
    RPL_LUSERME = 255,
    RPL_ADMINME = 256,
    RPL_ADMINLOC1 = 257,
    RPL_ADMINLOC2 = 258,
    RPL_ADMINEMAIL = 259,
    RPL_TRYAGAIN = 263,

    // errors
    ERR_NOSUCHNICK = 401, // No such nick/channel
    ERR_NOSUCHSERVER = 402, // No such server
    ERR_NOSUCHCHANNEL = 403, // No such channel
    ERR_CANNOTSENDTOCHAN = 404, // Cannot send to channel
    ERR_TOOMANYCHANNELS = 405, // You have joined too many channels
    ERR_WASNOSUCHNICK = 406, // There was no such nickname
    ERR_TOOMANYTARGETS = 407, // Duplicate recipients. No message delivered
    ERR_NOSUCHSERVICE = 408, // No such service
    ERR_NOORIGIN = 409, // No origin specified
    ERR_NORECIPIENT = 411, // No recipient given
    ERR_NOTEXTTOSEND = 412, // No text to send
    ERR_NOTOPLEVEL = 413, // No toplevel domain specified
    ERR_WILDTOPLEVEL = 414, // Wildcard in toplevel domain
    ERR_BADMASK = 415, // Bad server/host mask
    ERR_UNKNOWNCOMMAND = 421, // Unknown command
    ERR_NOMOTD = 422, // MOTD file is missing
    ERR_NOADMININFO = 423, // No administrative info available
    ERR_FILEERROR = 424, // File error
    ERR_NONICKNAMEGIVEN = 431, // No nickname given
    ERR_ERRONEUSNICKNAME = 432, // Erroneus nickname
    ERR_NICKNAMEINUSE = 433, // Nickname is already in use
    ERR_NICKCOLLISION = 436, // Nickname collision KILL
    ERR_UNAVAILRESOURCE = 437, // Nick/channel is temporarily unavailable
    ERR_USERNOTINCHANNEL = 441, // They aren't on that channel
    ERR_NOTONCHANNEL = 442, // You're not on that channel
    ERR_USERONCHANNEL = 443, // User is already on channel
    ERR_NOLOGIN = 444, // User not logged in
    ERR_SUMMONDISABLED = 445, // SUMMON has been disabled
    ERR_USERSDISABLED = 446, // USERS has been disabled
    ERR_NOTREGISTERED = 451, // You have not registered
    ERR_NEEDMOREPARAMS = 461, // Not enough parameters
    ERR_ALREADYREGISTERED = 462, // You may not reregister
    ERR_NOPERMFORHOST = 463, // Your host isn't among the privileged
    ERR_PASSWDMISMATCH = 464, // Password incorrect
    ERR_YOUREBANNEDCREEP = 465, // You are banned from this server
    ERR_YOUWILLBEBANNED = 466,
    ERR_KEYSET = 467, // Channel key already set
    ERR_CHANNELISFULL = 471, // Cannot join channel (+l)
    ERR_UNKNOWNMODE = 472, // Unknown mode char
    ERR_INVITEONLYCHAN = 473, // Cannot join channel (+i)
    ERR_BANNEDFROMCHAN = 474, // Cannot join channel (+b)
    ERR_BADCHANNELKEY = 475, // Cannot join channel (+k)
    ERR_BADCHANMASK = 476, // Bad channel mask
    ERR_NOCHANMODES = 477, // Channel doesn't support modes
    ERR_BANLISTFULL = 478, // Channel list is full
    ERR_NOPRIVILEGES = 481, // Permission denied- You're not an IRC operator
    ERR_CHANOPRIVSNEEDED = 482, // You're not channel operator
    ERR_CANTKILLSERVER = 483, // You can't kill a server!
    ERR_RESTRICTED = 484, // Your connection is restricted!
    ERR_UNIQOPPRIVSNEEDED = 485, // You're not the original channel operator
    ERR_NOOPERHOST = 491, // No O-lines for your host
    ERR_UMODEUNKNOWNFLAG = 501, // Unknown MODE flag
    ERR_USERSDONTMATCH = 502, // Can't change mode for other users

    //unused
    RPL_SERVICEINFO = 231,
    RPL_ENDOFSERVICES = 232,
    RPL_SERVICE = 233,
    RPL_NONE = 300,
    RPL_WHOISCHANOP = 316,
    RPL_KILLDONE = 361,
    RPL_CLOSING = 362,
    RPL_CLOSEEND = 363,
    RPL_INFOSTART = 373,
    RPL_MYPORTIS = 384,
    RPL_STATSCLINE = 213,
    RPL_STATSNLINE = 214,
    RPL_STATSILINE = 215,
    RPL_STATSKLINE = 216,
    RPL_STATSQLINE = 217,
    RPL_STATSYLINE = 218,
    RPL_STATSVLINE = 240,
    RPL_STATSLLINE = 241,
    RPL_STATSHLINE = 244,
    RPL_STATSPING = 246,
    RPL_STATSBLINE = 247,
    RPL_STATSDLINE = 250,
    ERR_NOSERVICEHOST = 492,

    // guesses
    RPL_TOPICDATE = 333, // date the topic was set, in seconds since the epoch
    ERR_MSGFORBIDDEN = 505, // freenode blocking privmsg from unreged users
}

#[deriving(PartialEq, Eq, Show)]
enum Reply {
    KnownReply(ReplyId),
    UnknownReply(i32),
}

impl FromStr for Reply {
    fn from_str (s: &str) -> Option<Reply> {
        // XXX
        match from_str(s) {
            Some(i) => Some(UnknownReply(i)),
            None => None,
        }
    }
}

#[deriving(PartialEq, Eq, Show)]
pub enum MessageType {
    CommandMessage(Command),
    ReplyMessage(Reply),
}

impl FromStr for MessageType {
    fn from_str (s: &str) -> Option<MessageType> {
        match s.char_at(0) {
            '0' .. '9' => {
                match from_str(s) {
                    Some(r) => Some(ReplyMessage(r)),
                    None => None,
                }
            },
            _ => {
                match from_str(s) {
                    Some(c) => Some(CommandMessage(c)),
                    None => None,
                }
            },
        }
    }
}

pub static MAX_MESSAGE_LENGTH: i32 = 512;
