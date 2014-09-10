use constants::{MessageType, MAX_MESSAGE_LENGTH};

use std::io;

#[deriving(PartialEq, Eq, Show)]
pub struct Message {
    from: Option<String>,
    message_type: MessageType,
    params: Vec<String>,
}

impl Message {
    pub fn new (from: Option<String>, message_type: MessageType, params: Vec<String>) -> Message {
        Message { from: from, message_type: message_type, params: params }
    }

    pub fn parse (msg: &str) -> Result<Message, &'static str> {
        if msg.len() > MAX_MESSAGE_LENGTH {
            return Err("message too long");
        }

        let message_parser = regex!(r"^(?::([^ ]+) )?([A-Z]+|[0-9]{3}) ([^\r\n\0]*)\r\n$");
        match message_parser.captures(msg) {
            Some(captures) => {
                let from = captures.at(1);
                let from = if from.len() > 0 { Some(from.to_string()) } else { None };
                let command = captures.at(2);

                let params = Message::parse_params(captures.at(3));

                match from_str(command) {
                    Some(c) => Ok(Message::new(from, c, params)),
                    None => Err("command parsing failed"),
                }
            },
            None => Err("message parsing failed"),
        }
    }

    pub fn from (&self) -> &Option<String> {
        &self.from
    }

    pub fn message_type (&self) -> &MessageType {
        &self.message_type
    }

    pub fn params (&self) -> &Vec<String> {
        &self.params
    }

    pub fn is_reply (&self) -> bool {
        self.message_type.is_reply()
    }

    pub fn write_protocol_string<W: Writer> (&self, w: &mut W) -> io::IoResult<()> {
        let mut buf = [0u8, ..MAX_MESSAGE_LENGTH];

        {
            let mut bufw = io::BufWriter::new(buf);

            match self.from {
                Some(ref f) => { try!(write!(bufw, ":{} ", f)) },
                None => {},
            }

            try!(write!(bufw, "{}", self.message_type));

            for param in self.params.iter() {
                if param.as_slice().contains_char(' ') {
                    try!(write!(bufw, " :{}", param));
                }
                else {
                    try!(write!(bufw, " {}", param));
                }
            }

            try!(write!(bufw, "\r\n"));
        }

        let len = buf.iter().position(|&c| c == 0).unwrap_or(MAX_MESSAGE_LENGTH);
        try!(w.write(buf.slice(0, len)));
        try!(w.flush());

        Ok(())
    }

    pub fn to_protocol_string (&self) -> String {
        let mut w = io::MemWriter::new();
        // this should never fail, so unwrap is fine
        self.write_protocol_string(&mut w).unwrap();
        // XXX encoding
        String::from_utf8_lossy(w.unwrap().as_slice()).into_string()
    }

    fn parse_params(params: &str) -> Vec<String> {
        let mut offset = 0;
        let len = params.len();
        let mut ret = vec![];

        loop {
            if offset >= len {
                return ret;
            }

            if params.char_at(offset) == ':' {
                ret.push(params.slice(offset + 1, len).to_string());
                return ret;
            }

            let remaining = params.slice(offset, len);
            match remaining.find(' ') {
                Some(next) => {
                    ret.push(remaining.slice(0, next).to_string());
                    offset += next + 1;
                },
                None => {
                    ret.push(remaining.to_string());
                    return ret;
                }
            }
        }
    }
}

#[test]
fn test_message_parser () {
    use constants::*;

    {
        let msg = "PASS secretpasswordhere\r\n";
        assert_eq!(
            Message::parse(msg),
            Ok(
                Message {
                    from: None,
                    message_type: Pass,
                    params: vec!["secretpasswordhere".to_string()],
                }
            )
        );
    }

    {
        let msg = ":WiZ NICK Kilroy\r\n";
        assert_eq!(
            Message::parse(msg),
            Ok(
                Message {
                    from: Some("WiZ".to_string()),
                    message_type: Nick,
                    params: vec!["Kilroy".to_string()],
                }
            )
        );
    }

    {
        let msg = "QUIT :Gone to have lunch\r\n";
        assert_eq!(
            Message::parse(msg),
            Ok(
                Message {
                    from: None,
                    message_type: Quit,
                    params: vec!["Gone to have lunch".to_string()],
                }
            )
        );
    }

    {
        let msg = ":Trillian SQUIT cm22.eng.umd.edu :Server out of control\r\n";
        assert_eq!(
            Message::parse(msg),
            Ok(
                Message {
                    from: Some("Trillian".to_string()),
                    message_type: Squit,
                    params: vec![
                        "cm22.eng.umd.edu".to_string(),
                        "Server out of control".to_string(),
                    ],
                }
            )
        );
    }

    {
        let msg = "401 doy :No such nick/channel\r\n";
        assert_eq!(
            Message::parse(msg),
            Ok(
                Message {
                    from: None,
                    message_type: Reply(ERR_NOSUCHNICK),
                    params: vec![
                        "doy".to_string(),
                        "No such nick/channel".to_string(),
                    ],
                }
            )
        );
    }
}
