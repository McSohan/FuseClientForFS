use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
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
        let uid = unsafe { libc::getuid() } as u32;
        let gid = unsafe { libc::getgid() } as u32;
        let pid = std::process::id(); // u32

        Self {
            len: (std::mem::size_of::<FuseInHeader>() + payload_len) as u32,
            opcode,
            unique,
            nodeid,
            uid,
            gid,
            pid,
            padding: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct FuseOutHeader {
    pub len: u32,
    pub error: i32,
    pub unique: u64,
}

impl FuseOutHeader {
    pub fn parse(buf: &[u8]) -> std::io::Result<(Self, &[u8])> {
        if buf.len() < std::mem::size_of::<Self>() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "short fuse_out_header",
            ));
        }

        let hdr =
            *bytemuck::from_bytes::<FuseOutHeader>(&buf[..std::mem::size_of::<FuseOutHeader>()]);

        let payload = &buf[std::mem::size_of::<Self>()..hdr.len as usize];
        Ok((hdr, payload))
    }
}
