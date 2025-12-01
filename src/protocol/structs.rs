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
            major: 7,            // Kernel-major protocol version
            minor: 31,           // Minor version used widely; 31-36 OK
            max_readahead: 0x20000,
            flags: 0,            // No flags requested
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

        let out = unsafe {
            *(buf.as_ptr() as *const FuseInitOut)
        };

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

        let out = unsafe {
            *(buf.as_ptr() as *const FuseEntryOut)
        };

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


