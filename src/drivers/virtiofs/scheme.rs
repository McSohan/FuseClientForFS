//! Scheme interface: userspace daemon speaks FUSE to kernel

use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};

use syscall::error::{Error, Result, EBADF};
use syscall::scheme::Scheme;

use super::device::VirtioFsDevice;

static mut REQUEST_BUF: Vec<u8> = Vec::new();

pub struct VirtioFsScheme<'a> {
    dev: Arc<VirtioFsDevice<'a>>,
    files: BTreeMap<usize, ()>,
    next_fd: usize,
}

impl<'a> VirtioFsScheme<'a> {
    pub fn new(dev: Arc<VirtioFsDevice<'a>>) -> Self {
        Self {
            dev,
            files: BTreeMap::new(),
            next_fd: 1,
        }
    }
}

impl<'a> Scheme for VirtioFsScheme<'a> {
    fn open(&mut self, _path: &[u8], _flags: usize, _ctx: usize) -> Result<usize> {
        let fd = self.next_fd;
        self.next_fd += 1;

        self.files.insert(fd, ());
        Ok(fd)
    }

    fn close(&mut self, fd: usize) -> Result<usize> {
        self.files.remove(&fd).ok_or(Error::new(EBADF))?;
        Ok(0)
    }

    fn write(&mut self, fd: usize, buf: &[u8], _offset: usize, _ctx: usize) -> Result<usize> {
        if !self.files.contains_key(&fd) {
            return Err(Error::new(EBADF));
        }

        unsafe {
            REQUEST_BUF.clear();
            REQUEST_BUF.extend_from_slice(buf);
        }

        Ok(buf.len())
    }

    fn read(&mut self, fd: usize, out: &mut [u8], _offset: usize, _ctx: usize) -> Result<usize> {
        if !self.files.contains_key(&fd) {
            return Err(Error::new(EBADF));
        }

        let req = unsafe { &REQUEST_BUF };

        let mut resp = vec![0u8; out.len()];
        let written = self.dev.send_request(&req, &mut resp)?;

        out[..written].copy_from_slice(&resp[..written]);

        Ok(written)
    }
}
