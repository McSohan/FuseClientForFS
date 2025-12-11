
use crate::protocol::FuseProtocol;

pub enum VirtiofsResource {
    File {
        inode: u64,
        fh: u64,
        offset: u64,
        flags: u32,
        path: String,
    },

    Dir {
        inode: u64,
        fh: u64,
        offset: u64,
        entries: Vec<(String, u64)>, // (name, inode)
        path: String,
    },
}

impl VirtiofsResource {
    pub fn inode(&self) -> u64 {
        match self {
            VirtiofsResource::File { inode, .. } => *inode,
            VirtiofsResource::Dir  { inode,  .. } => *inode,
        }
    }

    pub fn path(&self) -> &str {
        match self {
            VirtiofsResource::File { path, .. } => path,
            VirtiofsResource::Dir { path, .. } => path,
        }
    }
}
