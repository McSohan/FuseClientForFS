//! Transport wrapper used by the userspace virtiofs daemon

use alloc::sync::Arc;
use core::fmt;

use syscall::error::{Error, Result, EINVAL};

use super::device::VirtioFsDevice;

pub struct VirtioFsKernelTransport<'a> {
    dev: Arc<VirtioFsDevice<'a>>,
}

impl<'a> VirtioFsKernelTransport<'a> {
    pub fn new(dev: Arc<VirtioFsDevice<'a>>) -> Self {
        Self { dev }
    }

    pub fn roundtrip(&self, req: &[u8], resp: &mut [u8]) -> Result<usize> {
        self.dev.send_request(req, resp)
    }
}

impl fmt::Debug for VirtioFsKernelTransport<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VirtioFsKernelTransport")
    }
}
