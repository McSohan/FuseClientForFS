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
