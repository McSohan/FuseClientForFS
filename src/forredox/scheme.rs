// Attempt to enable mount -t virtiofs: mymount/  .... on RedoxOS

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use redox_scheme::{scheme::SchemeSync, CallerCtx, OpenResult};
use syscall::error::{Error, Result, EBADF, ENOENT};
use syscall::flag::{O_RDONLY, O_DIRECTORY, O_STAT, O_TRUNC};
use syscall::schemev2::NewFdFlags;

use crate::protocol::FuseProtocol;
use crate::virtiofs::resource::VirtiofsResource;

pub struct VirtiofsScheme {
    proto: FuseProtocol,

    /// Global FD allocator (matches RedoxFS next_id)
    next_id: AtomicUsize,

    /// id → resource (like RedoxFS’s `files: BTreeMap<usize, Resource>`)
    files: BTreeMap<usize, VirtiofsResource>,
}

impl VirtiofsScheme {
    pub fn new(proto: FuseProtocol) -> Self {
        Self {
            proto,
            next_id: AtomicUsize::new(1),
            files: BTreeMap::new(),
        }
    }

    /// Just like RedoxFS does parent/path resolution inside internal open
    fn resolve_inode(&mut self, path: &str) -> Result<u64> {
        self.proto
            .lookup_path(path)
            .map_err(|_| Error::new(ENOENT))
    }
}

impl SchemeSync for VirtiofsScheme {

    fn open(&mut self, url: &str, flags: usize, _ctx: &CallerCtx) -> Result<OpenResult> {
        let inode = self.resolve_inode(url)?;

        // If opening a directory (Redox uses O_STAT | O_DIRECTORY etc.)
        if flags & O_DIRECTORY == O_DIRECTORY {
            let out = self.proto.opendir(inode)?;
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);

            // preload entries? optional. RedoxFS loads entries lazily
            let resource = VirtiofsResource::Dir {
                inode,
                fh: out.fh,
                offset: 0,
                entries: Vec::new(),
                path: url.to_string(),
            };

            self.files.insert(id, resource);

            return Ok(OpenResult::ThisScheme { number: id, flags: NewFdFlags::POSITIONED });
        }

        // Regular file
        let out = self.proto.open(inode, flags as u32)
            .map_err(|_| Error::new(ENOENT))?;

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        let resource = VirtiofsResource::File {
            inode,
            fh: out.fh,
            offset: 0,
            flags: flags as u32,
            path: url.to_string(),
        };

        self.files.insert(id, resource);

        Ok(OpenResult::ThisScheme { number: id, flags: NewFdFlags::POSITIONED })
    }

    fn read(
        &mut self,
        id: usize,
        buf: &mut [u8],
        offset: u64,
        _fcntl_flags: u32,
        _ctx: &CallerCtx,
    ) -> Result<usize> {

        let res = self.files.get_mut(&id).ok_or(Error::new(EBADF))?;

        match res {
            VirtiofsResource::File { inode, fh, offset: off, .. } => {
                let data = self.proto
                    .read(*inode, *fh, offset, buf.len() as u32)
                    .map_err(|_| Error::new(EBADF))?;

                buf[..data.len()].copy_from_slice(&data);
                *off = offset + data.len() as u64;
                Ok(data.len())
            }

            VirtiofsResource::Dir { inode, fh, offset: off, entries, .. } => {
                // Load directory entries lazily
                if entries.is_empty() {
                    let ents = self.proto.readdir(*inode, *fh, 0, 4096)
                        .map_err(|_| Error::new(EBADF))?;

                    for e in ents {
                        entries.push((e.name.clone(), e.ino));
                    }
                }

                // Similar to RedoxFS: convert entries to DirentBuf
                let mut written = 0;
                for (name, ino) in entries.iter() {
                    let line = format!("{name}\0");
                    let bytes = line.as_bytes();
                    if written + bytes.len() > buf.len() { break; }
                    buf[written..written+bytes.len()].copy_from_slice(bytes);
                    written += bytes.len();
                }
                Ok(written)
            }
        }
    }

    fn write(
        &mut self,
        id: usize,
        buf: &[u8],
        offset: u64,
        _fcntl_flags: u32,
        _ctx: &CallerCtx,
    ) -> Result<usize> {

        let res = self.files.get_mut(&id).ok_or(Error::new(EBADF))?;

        match res {
            VirtiofsResource::File { inode, fh, .. } => {
                let written = self.proto
                    .write(*inode, *fh, offset, buf)
                    .map_err(|_| Error::new(EBADF))?;
                Ok(written)
            }
            _ => Err(Error::new(EISDIR)),
        }
    }

        fn fstat(&mut self, id: usize, stat: &mut Stat, _ctx: &CallerCtx) -> Result<()> {
        let res = self.files.get(&id).ok_or(Error::new(EBADF))?;
        let inode = res.inode();

        let attr = self.proto.getattr(inode)
            .map_err(|_| Error::new(EBADF))?
            .attr;

        stat.st_ino = inode;
        stat.st_size = attr.size;
        stat.st_mode = attr.mode;
        stat.st_uid = attr.uid;
        stat.st_gid = attr.gid;
        stat.st_blocks = attr.blocks;
        stat.st_blksize = attr.blksize;
        stat.st_atime = attr.atime;
        stat.st_mtime = attr.mtime;
        stat.st_ctime = attr.ctime;

        Ok(())
    }

        fn on_close(&mut self, id: usize) {
        if let Some(res) = self.files.remove(&id) {
            match res {
                VirtiofsResource::File { inode, fh, .. } => {
                    let _ = self.proto.release(inode, fh);
                }
                VirtiofsResource::Dir { inode, fh, .. } => {
                    let _ = self.proto.releasedir(inode, fh);
                }
            }
        }
    }
}

