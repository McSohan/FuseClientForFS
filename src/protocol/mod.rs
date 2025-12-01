// FUSE protocol manager (unique counter, send/recv)

/*
pub struct FuseProtocol<T: Transport> {
    pub transport: T,
    pub unique: u64,
}

impl<T: Transport> FuseProtocol<T> {
    pub fn new(transport: T) -> Self {
        Self { transport, unique: 2 }
    }

    pub fn send_request(
        &mut self,
        opcode: u32,
        nodeid: u64,
        payload: &[u8],
    ) -> std::io::Result<(FuseOutHeader, Vec<u8>)> {
        let unique = self.alloc_unique();
        let header = FuseInHeader::new(opcode, nodeid, unique, payload.len());
        let mut buf = header.to_bytes();
        buf.extend_from_slice(payload);

        self.transport.send(&buf)?;
        let resp = self.transport.recv()?;

        let (hdr, payload) = FuseOutHeader::parse(&resp)?;
        Ok((hdr, payload))
    }

    fn alloc_unique(&mut self) -> u64 {
        let u = self.unique;
        self.unique += 1;
        u
    }
}
    */


use crate::transport::unix_socket::FuseStream;

pub struct FuseProtocol {
    stream: FuseStream,
}

impl FuseProtocol {
    pub fn new(stream: FuseStream) -> Self {
        Self { stream }
    }

    pub fn send_init(&mut self) -> std::io::Result<()> {
        const FUSE_INIT_MSG: [u8; 104] = [
            // header (40 bytes)
            104, 0, 0, 0,     // len = 104
            26, 0, 0, 0,      // opcode = INIT
            2, 0, 0, 0,       // unique = 2
            0, 0, 0, 0,       // unique high
            0, 0, 0, 0,       // nodeid
            0, 0, 0, 0,       // uid
            0, 0, 0, 0,       // gid
            0, 0, 0, 0,       // pid
            0, 0, 0, 0,       // padding

            // payload (64 bytes)
            7, 0, 0, 0,
            41, 0, 0, 0,
            0, 0, 2, 0,
            251, 255, 255, 115,
            255, 1, 0, 0,

            // rest zero
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
        ];

        self.stream.send(&FUSE_INIT_MSG)?;
        Ok(())
    }
}
