use std::io;
use std::os::unix::net::{UnixListener, UnixStream};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

use crate::transport::common::FuseTransport;

pub struct FuseListener {
    listener: UnixListener,
}

pub struct FuseStream {
    stream: UnixStream,
}

impl FuseListener {
    pub fn bind(path: &str) -> std::io::Result<Self> {
        let p = Path::new(path);
        if p.exists() {
            fs::remove_file(p)?;
        }
        let listener = UnixListener::bind(p)?;
        Ok(Self { listener })
    }

    pub fn accept(&self) -> std::io::Result<FuseStream> {
        let (stream, _) = self.listener.accept()?;
        Ok(FuseStream { stream })
    }
}

impl FuseStream {
    pub fn send(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.stream.write_all(data)?;
        self.stream.flush()
    }

    pub fn recv_raw(&mut self) -> std::io::Result<Vec<u8>> {
        // read length
        let mut header = [0u8; 4];
        self.stream.read_exact(&mut header)?;

        let len = u32::from_le_bytes(header) as usize;

        let mut buf = vec![0u8; len];
        buf[..4].copy_from_slice(&header);
        self.stream.read_exact(&mut buf[4..])?;

        Ok(buf)
    }
}

impl FuseTransport for FuseStream {
    fn roundtrip(&mut self, req: &[u8]) -> io::Result<Vec<u8>> {
        self.send(req)?;
        self.recv_raw()
    }
}
