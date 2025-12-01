#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FuseInHeader {
    pub len: u32,
    pub opcode: u32,
    pub unique: u64,
    pub nodeid: u64,
    pub uid: u32,
    pub gid: u32,
    pub pid: u32,
    pub padding: u32,
}

impl FuseInHeader {
    pub fn new(opcode: u32, nodeid: u64, unique: u64, payload_len: usize) -> Self {
        Self {
            len: (std::mem::size_of::<FuseInHeader>() + payload_len) as u32,
            opcode,
            unique,
            nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            padding: 0,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>(),
            )
            .to_vec()
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FuseOutHeader {
    pub len: u32,
    pub error: i32,
    pub unique: u64,
}

impl FuseOutHeader {
    pub fn parse(buf: &[u8]) -> std::io::Result<(Self, &[u8])> {
        if buf.len() < std::mem::size_of::<Self>() {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "short fuse_out_header"));
        }

        let hdr = unsafe {
            *(buf.as_ptr() as *const FuseOutHeader)
        };

        let payload = &buf[std::mem::size_of::<Self>()..hdr.len as usize];
        Ok((hdr, payload))
    }
}
