// FUSE protocol manager (unique counter, send/recv)

mod headers;
// Todo: make this not pub
// This shouldnt be pub technically, but,
// I just want to get rid of the warnings for now - when I do cargo check
pub mod opcodes;
mod structs;

use self::headers::*;
use self::opcodes::*;
use self::structs::*;
use crate::transport::unix_socket::FuseStream;

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

    fn errno_name(code: i32) -> &'static str {
        match code {
            libc::ENOENT => "ENOENT (No such file or directory)",
            libc::EACCES => "EACCES (Permission denied)",
            libc::EEXIST => "EEXIST (File exists)",
            libc::ENOSYS => "ENOSYS (Function not implemented)",
            libc::EROFS => "EROFS (Read-only filesystem)",
            libc::ENOTDIR => "ENOTDIR (Not a directory)",
            libc::EISDIR => "EISDIR (Is a directory)",
            libc::EINVAL => "EINVAL (Invalid argument)",
            libc::EPERM => "EPERM (Operation not permitted)",
            libc::ENOTEMPTY => "ENOTEMPTY (Directory not empty)",
            _ => "Unknown error",
        }
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

        if out_hdr.error != 0 {
            let errno = -out_hdr.error;
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{} ({})", Self::errno_name(errno), errno),
            ));
        }

        // 6) Return header + payload
        Ok((out_hdr, payload_bytes.to_vec()))
    }

    pub fn send_init(&mut self) -> std::io::Result<FuseInitOut> {
        let init_in = FuseInitIn::new();
        let payload = init_in.as_bytes();

        let (hdr, payload_bytes) = self.send_request(FUSE_INIT, 0, payload)?;

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

    pub fn lookup(&mut self, parent: u64, name: &str) -> std::io::Result<FuseEntryOut> {
        // FUSE LOOKUP requires utf8 bytes + trailing null byte
        let mut payload = name.as_bytes().to_vec();
        payload.push(0);

        // Send the request
        let (hdr, resp_payload) = self.send_request(FUSE_LOOKUP, parent, &payload)?;

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

    pub fn open(&mut self, nodeid: u64, flags: u32) -> std::io::Result<FuseOpenOut> {
        let input = FuseOpenIn::new(flags);
        let payload = input.as_bytes();

        let (hdr, resp_payload) = self.send_request(FUSE_OPEN, nodeid, payload)?;

        if hdr.error != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("OPEN failed: {}", hdr.error),
            ));
        }

        let out = FuseOpenOut::parse(&resp_payload)?;
        Ok(out)
    }

    pub fn read(
        &mut self,
        nodeid: u64,
        fh: u64,
        offset: u64,
        size: u32,
    ) -> std::io::Result<Vec<u8>> {
        let req = FuseReadIn {
            fh,
            offset,
            size,
            read_flags: 0,
            lock_owner: 0,
            flags: 0,
            padding: 0,
        };

        let payload = unsafe {
            std::slice::from_raw_parts(
                &req as *const _ as *const u8,
                std::mem::size_of::<FuseReadIn>(),
            )
        };

        let (hdr, data) = self.send_request(FUSE_READ, nodeid, payload)?;

        if hdr.error != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("READ failed with error {}", hdr.error),
            ));
        }

        Ok(data)
    }

    pub fn release(&mut self, inode: u64, fh: u64) -> std::io::Result<()> {
        // Build fuse_release_in
        let release_in = FuseReleaseIn {
            fh,
            flags: 0,
            release_flags: 0,
            lock_owner: 0,
        };

        // SAFELY reinterpret struct as bytes
        let bytes = unsafe {
            std::slice::from_raw_parts(
                &release_in as *const FuseReleaseIn as *const u8,
                std::mem::size_of::<FuseReleaseIn>(),
            )
        };

        // Send request — reply has no payload
        let (hdr, _) = self.send_request(FUSE_RELEASE, inode, bytes)?;

        if hdr.error != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("RELEASE failed with error {}", hdr.error),
            ));
        }

        Ok(())
    }

    pub fn getattr(&mut self, nodeid: u64) -> std::io::Result<FuseAttrOut> {
        let inmsg = FuseGetattrIn::new();
        let payload = inmsg.as_bytes();

        let (hdr, resp) = self.send_request(FUSE_GETATTR, nodeid, payload)?;

        if hdr.error != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("GETATTR failed with error {}", hdr.error),
            ));
        }

        let out = FuseAttrOut::parse(&resp)?;
        Ok(out)
    }

    pub fn readdir(
        &mut self,
        nodeid: u64,
        fh: u64,
        offset: u64,
        size: u32,
    ) -> std::io::Result<Vec<DirEntry>> {
        let req = FuseReadIn {
            fh,
            offset,
            size,
            read_flags: 0,
            lock_owner: 0,
            flags: 0,
            padding: 0,
        };

        let payload = unsafe {
            std::slice::from_raw_parts(
                &req as *const FuseReadIn as *const u8,
                std::mem::size_of::<FuseReadIn>(),
            )
        };

        let (hdr, data) = self.send_request(FUSE_READDIR, nodeid, payload)?;

        if hdr.error != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("READDIR failed: {}", hdr.error),
            ));
        }

        if data.is_empty() {
            return Ok(Vec::new());
        }

        DirEntry::parse_dirents(&data)
    }

    pub fn mkdir(&mut self, parent: u64, name: &str, mode: u32) -> std::io::Result<FuseEntryOut> {
        // Step 1: build mkdir_in
        let mk = FuseMkdirIn::new(mode, 0);

        // Step 2: build payload = mk | name\0
        let mut payload = mk.as_bytes().to_vec();
        payload.extend_from_slice(name.as_bytes());
        payload.push(0);

        // Step 3: send
        let (hdr, resp) = self.send_request(FUSE_MKDIR, parent, &payload)?;

        if hdr.error != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("MKDIR failed with error {}", hdr.error),
            ));
        }

        // Step 4: parse entry
        let entry = FuseEntryOut::parse(&resp)?;
        Ok(entry)
    }

    pub fn releasedir(&mut self, nodeid: u64, fh: u64) -> std::io::Result<()> {
        let input = FuseReleaseIn {
            fh,
            flags: 0,
            release_flags: 0,
            lock_owner: 0,
        };

        let payload = unsafe {
            std::slice::from_raw_parts(
                &input as *const FuseReleaseIn as *const u8,
                std::mem::size_of::<FuseReleaseIn>(),
            )
        };

        let (hdr, _) = self.send_request(FUSE_RELEASEDIR, nodeid, payload)?;

        if hdr.error != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("RELEASEDIR failed: {}", hdr.error),
            ));
        }

        Ok(())
    }

    pub fn opendir(&mut self, nodeid: u64) -> std::io::Result<FuseOpenOut> {
        // OPENDIR uses FuseOpenIn exactly like OPEN
        let input = FuseOpenIn::new(libc::O_RDONLY as u32);
        let payload = input.as_bytes();

        let (hdr, resp_payload) = self.send_request(FUSE_OPENDIR, nodeid, payload)?;

        if hdr.error != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("OPENDIR failed: {}", hdr.error),
            ));
        }

        let out = FuseOpenOut::parse(&resp_payload)?;
        Ok(out)
    }
}
