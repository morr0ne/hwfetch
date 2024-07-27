use std::{
    fs::File,
    io::{BufRead, BufReader, Error as IoError},
};

use rustix::process::{Gid, Uid};

pub fn getpwuid(uid: Uid) -> Option<Passwd> {
    let mut pw = None;

    let mut parser = Parser::new().unwrap();

    while let Some(entry) = parser.next_entry().unwrap() {
        if entry.uid == uid {
            pw = Some(entry)
        }
    }

    pw
}

#[allow(unused)]
#[derive(Debug)]
pub struct Passwd {
    pub name: String,
    pub passwd: String,
    pub uid: Uid,
    pub gid: Gid,
    pub gecos: String,
    pub dir: String,
    pub shell: String,
}

pub struct Parser<R> {
    reader: R,
    buf: String,
}

impl Parser<BufReader<File>> {
    pub fn new() -> Result<Self, IoError> {
        let file = File::open("/etc/passwd")?;
        let reader = BufReader::new(file);

        Ok(Self {
            reader,
            buf: String::new(),
        })
    }

    pub fn next_entry(&mut self) -> Result<Option<Passwd>, IoError> {
        self.buf.clear();

        self.reader.read_line(&mut self.buf)?;

        Ok(Passwd::from_buf(&self.buf))
    }
}

impl Passwd {
    fn from_buf(buf: &str) -> Option<Self> {
        let mut entries = buf.splitn(7, |s| s == ':');

        let name = entries.next()?.to_string();
        let passwd = entries.next()?.to_string();

        let uid = entries
            .next()?
            .parse()
            .map(|n| unsafe { Uid::from_raw(n) })
            .ok()?;

        let gid = entries
            .next()?
            .parse()
            .map(|n| unsafe { Gid::from_raw(n) })
            .ok()?;

        let gecos = entries.next()?.to_string();
        let dir = entries.next()?.to_string();
        let shell = entries.next()?.to_string();

        Some(Passwd {
            name,
            passwd,
            uid,
            gid,
            gecos,
            dir,
            shell,
        })
    }
}
