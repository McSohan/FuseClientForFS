use bytemuck::{Pod, Zeroable};
use bytemuck::try_from_bytes;
use std::mem::size_of;

// #[repr(C)] FUSE payload structs
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
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


}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
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

        let out = bytemuck::from_bytes::<FuseInitOut>(&buf[..]);

        Ok(*out)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
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
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
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

        let size = std::mem::size_of::<FuseEntryOut>();
        let out = *bytemuck::from_bytes::<FuseEntryOut>(&buf[..size]);

        Ok(out)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct FuseOpenIn {
    pub flags: u32, // these will come from lbc -- O_RDONLY, O_WRONLY etc.
    pub unused: u32,
}

impl FuseOpenIn {
    pub fn new(flags: u32) -> Self {
        Self { flags, unused: 0 }
    }

}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
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

        Ok(*bytemuck::from_bytes::<FuseOpenOut>(&buf))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
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
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct FuseReleaseIn {
    pub fh: u64,
    pub flags: u32,
    pub release_flags: u32,
    pub lock_owner: u64,
}

#[repr(C)]
#[derive( Clone, Copy, Pod, Zeroable)]
pub struct FuseMkdirIn {
    pub mode: u32,
    pub umask: u32,
}

impl FuseMkdirIn {
    pub fn new(mode: u32, umask: u32) -> Self {
        Self { mode, umask }
    }

    
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
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
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct FuseAttrOut {
    pub attr_valid: u64,
    pub attr_valid_nsec: u32,
    pub dummy: u32,
    pub attr: FuseAttr,
}

impl FuseAttrOut {
    pub fn parse(buf: &[u8]) -> std::io::Result<Self> {
        let needed = std::mem::size_of::<Self>();

        if buf.len() < needed {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("FuseAttrOut too small: got {}, need {}", buf.len(), needed),
            ));
        }

        // Safe reinterpretation
        let out: &Self = bytemuck::try_from_bytes(&buf[..needed])
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "FuseAttrOut not properly aligned or POD",
                )
            })?;

        Ok(*out) // copy out
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

        let U64 = size_of::<u64>();
        let U32 = size_of::<u32>();

        // struct fuse_dirent {
        //   u64 ino;
        //   u64 off;
        //   u32 namelen;
        //   u32 type;
        //   char name[];
        // }  // then 8-byte aligned
        let dirent_hdr_size: usize = U64 + U64 + U32 + U32; // 24

        while pos + dirent_hdr_size <= buf.len() {
            // ---- Read fixed header fields ----
            let ino = u64::from_le_bytes(buf[pos..pos + U64].try_into().unwrap());
            pos += U64;

            let off = u64::from_le_bytes(buf[pos..pos + U64].try_into().unwrap());
            pos += U64;

            let namelen = u32::from_le_bytes(buf[pos..pos + U32].try_into().unwrap());
            pos += U32;

            let typ = u32::from_le_bytes(buf[pos..pos + U32].try_into().unwrap());
            pos += U32;

            // Now pos = start of the name field
            let name_start = pos;
            let name_end = name_start + namelen as usize;

            if name_end > buf.len() {
                break; // corrupted / truncated entry
            }

            let name = String::from_utf8_lossy(&buf[name_start..name_end]).to_string();

            entries.push(DirEntry {
                ino,
                offset: off,
                namelen,
                typ,
                name,
            });

            pos = name_end;

            // ---- Record alignment ----
            // rec_len = ALIGN( hdr_size + namelen )
            let rec_len = (dirent_hdr_size + namelen as usize + 7) & !7;

            pos = pos - dirent_hdr_size; // rewind to start of this record
            pos += rec_len; // skip whole aligned record
        }

        Ok(entries)
    }
}
