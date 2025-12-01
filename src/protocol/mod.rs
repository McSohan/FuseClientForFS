// FUSE protocol manager (unique counter, send/recv)

pub mod headers;
pub mod opcodes;
pub mod structs;

use crate::transport::unix_socket::FuseStream;
use self::headers::*;
use self::structs::{FuseInitIn, FuseInitOut, FuseEntryOut};
use self::opcodes::*;


pub struct FuseProtocol {
    stream: FuseStream,
    next_unique: u64,
}

impl FuseProtocol {
    pub fn new(stream: FuseStream) -> Self {
        Self {
            stream,
            next_unique: 2,
        }
    }

    fn alloc_unique(&mut self) -> u64 {
        let u = self.next_unique;
        self.next_unique += 1;
        u
    }

    pub fn send_request(
        &mut self,
        opcode: u32,
        nodeid: u64,
        payload: &[u8],
    ) -> std::io::Result<(FuseOutHeader, Vec<u8>)> {
        // 1) Build fuse_in_header
        let unique = self.alloc_unique();
        let header = FuseInHeader::new(opcode, nodeid, unique, payload.len());

        // 2) Build the request buffer: header + payload
        let mut msg = header.to_bytes();
        msg.extend_from_slice(payload);

        // 3) Send raw data
        self.stream.send(&msg)?;

        // 4) Receive raw response
        let raw = self.stream.recv_raw()?;

        // 5) Parse fuse_out_header
        let (out_hdr, payload_bytes) = FuseOutHeader::parse(&raw)?;
        
        // 6) Return header + payload
        Ok((out_hdr, payload_bytes.to_vec()))
    }

    pub fn send_init(&mut self) -> std::io::Result<FuseInitOut> {
        let init_in = FuseInitIn::new();
        let payload = init_in.as_bytes();

        let (hdr, payload_bytes) =
            self.send_request(FUSE_INIT, 0, payload)?;

        if hdr.error != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("FUSE_INIT error {}", hdr.error),
            ));
        }

        // Parse the output
        let init_out = FuseInitOut::parse(&payload_bytes)?;

        println!(
            "FUSE INIT OK: daemon supports major={} minor={} max_write={} flags={:#x}",
            init_out.major, init_out.minor, init_out.max_write, init_out.flags
        );

        Ok(init_out)
    }

    pub fn lookup(&mut self, parent: u64, name: &str)
        -> std::io::Result<FuseEntryOut>
    {
        // FUSE LOOKUP requires utf8 bytes + trailing null byte
        let mut payload = name.as_bytes().to_vec();
        payload.push(0);

        // Send the request
        let (hdr, resp_payload) =
            self.send_request(FUSE_LOOKUP, parent, &payload)?;

        if hdr.error != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("LOOKUP failed with error {}", hdr.error),
            ));
        }

        // Parse fuse_entry_out from response payload
        let entry = FuseEntryOut::parse(&resp_payload)?;

        Ok(entry)
    }
}


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

/*
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

    */

