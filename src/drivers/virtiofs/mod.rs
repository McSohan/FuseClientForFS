//! VirtioFS kernel driver for Redox OS

#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;

use syscall::error::{Error, Result};

mod device;
mod transport;
mod scheme;

use device::virtiofs_probe;
use scheme::VirtioFsScheme;

/// Entry point called by the Redox kernel driver manager
pub fn init() -> Result<()> {
    log::info!("virtiofs: initializing driver");

    // Probe PCI device 0x105A
    let dev = match virtiofs_probe() {
        Ok(d) => Arc::new(d),
        Err(e) => {
            log::warn!("virtiofs: no device found: {:?}", e);
            return Err(e);
        }
    };

    log::info!("virtiofs: device probed, creating scheme");

    // Create userspace scheme interface
    let scheme = VirtioFsScheme::new(dev.clone());
    let handle = syscall::scheme::schemes_mut().new_scheme("virtiofs", Box::new(scheme))?;

    log::info!("virtiofs: registered scheme 'virtiofs' fd={}", handle);

    Ok(())
}
