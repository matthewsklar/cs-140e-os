use atags::raw;
use std::slice;
use std::str;

pub use atags::raw::{Core, Mem};

/// An ATAG.
#[derive(Debug, Copy, Clone)]
pub enum Atag {
    Core(raw::Core),
    Mem(raw::Mem),
    Cmd(&'static str),
    Unknown(u32),
    None
}

impl Atag {
    /// Returns `Some` if this is a `Core` ATAG. Otherwise returns `None`.
    pub fn core(self) -> Option<Core> {
        match self {
            Atag::Core(x) => Some(x),
            _ => None
        }
    }

    /// Returns `Some` if this is a `Mem` ATAG. Otherwise returns `None`.
    pub fn mem(self) -> Option<Mem> {
        match self {
            Atag::Mem(x) => Some(x),
            _ => None
        }
    }

    /// Returns `Some` with the command line string if this is a `Cmd` ATAG.
    /// Otherwise returns `None`.
    pub fn cmd(self) -> Option<&'static str> {
        match self {
            Atag::Cmd(x) => Some(x),
            _ => None
        }
    }
}

impl From<raw::Core> for Atag {
    fn from(kind: raw::Core) -> Atag {
        Atag::Core(kind)
    }
}

impl From<raw::Mem> for Atag {
    fn from(kind: raw::Mem) -> Atag {
        Atag::Mem(kind)
    }
}

impl<'a> From<&'a raw::Cmd> for Atag {
    fn from(kind: &raw::Cmd) -> Atag {
        let mut len : usize = 0;
        let base_addr = &kind.cmd as *const u8;

        unsafe {
            let mut addr = base_addr;
            while *addr != 0 {
                len += 1;
                addr = addr.offset(1);
            }

            let mem = slice::from_raw_parts(base_addr, len);
            Atag::Cmd(str::from_utf8(mem).unwrap())
        }
    }
}

impl<'a> From<&'a raw::Atag> for Atag {
    fn from(atag: &raw::Atag) -> Atag {
        unsafe {
            match (atag.tag, &atag.kind) {
                (raw::Atag::CORE, &raw::Kind { core }) =>
                    Atag::from(core),
                (raw::Atag::MEM, &raw::Kind { mem }) =>
                    Atag::from(mem),
                (raw::Atag::CMDLINE, &raw::Kind { ref cmd }) =>
                    Atag::from(cmd),
                (raw::Atag::NONE, _) => Atag::None,
                (id, _) => Atag::Unknown(id)
            }
        }
    }
}
