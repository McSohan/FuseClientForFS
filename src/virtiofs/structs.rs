pub type Fd = u32;

pub struct OpenFile {
    pub inode: u64,
    pub fh: u64,
    pub offset: u64,
    pub flags: u32,
}

pub struct FileStat {
    pub inode: u64,
    pub size: u64,
    pub mode: u32,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,

    pub blocks: u64,
    pub blksize: u32,
    pub rdev: u32,

    pub atime: u64,
    pub atime_nsec: u32,

    pub mtime: u64,
    pub mtime_nsec: u32,

    pub ctime: u64,
    pub ctime_nsec: u32,
}

pub struct DirEntryInfo {
    pub name: String,
    pub inode: u64,
    pub mode: u32,
}
