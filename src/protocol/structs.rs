// #[repr(C)] FUSE payload structs
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FuseInitIn {
    pub major: u32,
    pub minor: u32,
    pub max_readahead: u32,
    pub flags: u32,
}

impl FuseInitIn {
    pub fn new() -> Self {
        Self {
            major: 7,  // Kernel-major protocol version
            minor: 31, // Minor version used widely; 31-36 OK
            max_readahead: 0x20000,
            flags: 0, // No flags requested
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FuseInitOut {
    pub major: u32,
    pub minor: u32,
    pub max_readahead: u32,
    pub flags: u32,
    pub max_background: u16,
    pub congestion_threshold: u16,
    pub max_write: u32,
    // There may be more fields! We ignore optional ones for now.
}

impl FuseInitOut {
    pub fn parse(buf: &[u8]) -> std::io::Result<Self> {
        if buf.len() < std::mem::size_of::<Self>() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "fuse_init_out too small",
            ));
        }

        let out = unsafe { *(buf.as_ptr() as *const FuseInitOut) };

        Ok(out)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FuseAttr {
    pub ino: u64,
    pub size: u64,
    pub blocks: u64,

    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,

    pub atimensec: u32,
    pub mtimensec: u32,
    pub ctimensec: u32,

    pub mode: u32,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u32,
    pub blksize: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FuseEntryOut {
    pub nodeid: u64,
    pub generation: u64,

    pub entry_valid: u64,
    pub attr_valid: u64,

    pub entry_valid_nsec: u32,
    pub attr_valid_nsec: u32,

    pub attr: FuseAttr,
}

impl FuseEntryOut {
    pub fn parse(buf: &[u8]) -> std::io::Result<Self> {
        if buf.len() < std::mem::size_of::<Self>() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!(
                    "fuse_entry_out too small: got {} expected {}",
                    buf.len(),
                    std::mem::size_of::<Self>()
                ),
            ));
        }

        let out = unsafe { *(buf.as_ptr() as *const FuseEntryOut) };

        Ok(out)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FuseOpenIn {
    pub flags: u32, // these will come from lbc -- O_RDONLY, O_WRONLY etc.
    pub unused: u32,
}

impl FuseOpenIn {
    pub fn new(flags: u32) -> Self {
        Self { flags, unused: 0 }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const FuseOpenIn) as *const u8,
                std::mem::size_of::<FuseOpenIn>(),
            )
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FuseOpenOut {
    pub fh: u64,
    pub open_flags: u32,
    pub padding: u32,
}

impl FuseOpenOut {
    pub fn parse(buf: &[u8]) -> std::io::Result<Self> {
        // Should this be equal??? Maybe not because future versions can append shit to it.
        if buf.len() < std::mem::size_of::<Self>() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("FuseOpenOut too small: {} bytes", buf.len()),
            ));
        }

        Ok(unsafe { *(buf.as_ptr() as *const FuseOpenOut) })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FuseReadIn {
    pub fh: u64,
    pub offset: u64,
    pub size: u32,
    pub read_flags: u32,
    pub lock_owner: u64,
    pub flags: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FuseReleaseIn {
    pub fh: u64,
    pub flags: u32,
    pub release_flags: u32,
    pub lock_owner: u64,
}

#[repr(C)]
pub struct FuseMkdirIn {
    pub mode: u32,
    pub umask: u32,
}

impl FuseMkdirIn {
    pub fn new(mode: u32, umask: u32) -> Self {
        Self { mode, umask }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const FuseMkdirIn) as *const u8,
                std::mem::size_of::<FuseMkdirIn>(),
            )
        }
    }
}

// This impl is not required
/*
impl FuseReleaseIn {
    pub fn parse(buf: &[u8]) -> std::io::Result<Self> {
        if buf.len() < std::mem::size_of::<Self>() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "FuseReleaseIn too small",
            ));
        }

        let r = unsafe { *(buf.as_ptr() as *const FuseReleaseIn) };
        Ok(r)
    }
}
*/

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FuseGetattrIn {
    pub getattr_flags: u32,
    pub dummy: u32,
    pub fh: u64,
}

impl FuseGetattrIn {
    pub fn new() -> Self {
        Self {
            getattr_flags: 0,
            dummy: 0,
            fh: 0,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FuseAttrOut {
    pub attr_valid: u64,
    pub attr_valid_nsec: u32,
    pub dummy: u32,
    pub attr: FuseAttr,
}

impl FuseAttrOut {
    pub fn parse(buf: &[u8]) -> std::io::Result<Self> {
        if buf.len() < std::mem::size_of::<Self>() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("FuseAttrOut too small: got {}", buf.len()),
            ));
        }
        let out = unsafe { *(buf.as_ptr() as *const Self) };
        Ok(out)
    }
}

use std::io;

#[derive(Debug)]
pub struct DirEntry {
    pub ino: u64,
    pub offset: u64,
    pub namelen: u32,
    pub typ: u32,
    pub name: String,
}

impl DirEntry {
    pub fn parse_dirents(buf: &[u8]) -> io::Result<Vec<DirEntry>> {
        let mut entries = Vec::new();
        let mut pos = 0usize;

        // struct fuse_dirent {
        //   u64 ino;
        //   u64 off;
        //   u32 namelen;
        //   u32 type;
        //   char name[];
        // }  // then 8-byte aligned
        const DIRENT_HDR_SIZE: usize = 8 + 8 + 4 + 4; // 24

        while pos + DIRENT_HDR_SIZE <= buf.len() {
            let ino = u64::from_le_bytes(buf[pos..pos + 8].try_into().unwrap());
            let off = u64::from_le_bytes(buf[pos + 8..pos + 16].try_into().unwrap());
            let namelen = u32::from_le_bytes(buf[pos + 16..pos + 20].try_into().unwrap());
            let typ = u32::from_le_bytes(buf[pos + 20..pos + 24].try_into().unwrap());

            let name_start = pos + DIRENT_HDR_SIZE;
            let name_end = name_start + namelen as usize;
            if name_end > buf.len() {
                break;
            }

            let name = String::from_utf8_lossy(&buf[name_start..name_end]).to_string();

            entries.push(DirEntry {
                ino,
                offset: off,
                namelen,
                typ,
                name,
            });

            // FUSE_DIRENT_ALIGN(DIRENT_HDR_SIZE + namelen)
            let rec_len = (DIRENT_HDR_SIZE + namelen as usize + 7) & !7;
            pos += rec_len;
        }

        Ok(entries)
    }
}
