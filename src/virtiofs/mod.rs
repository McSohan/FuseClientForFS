mod structs;

use std::collections::HashMap;
use std::path::PathBuf;

use self::structs::{DirEntryInfo, Fd, FileStat, OpenFile};
use crate::protocol::FuseProtocol;

pub struct VirtioFsImpl {
    proto: FuseProtocol,
    cwd_inode: u64,
    cwd_path: PathBuf,
    next_fd: Fd,
    open_files: HashMap<Fd, OpenFile>,
}

impl VirtioFsImpl {
    pub fn new(proto: FuseProtocol) -> Self {
        Self {
            proto,
            cwd_inode: 1, // root inode in your FS
            cwd_path: PathBuf::from("/"),
            next_fd: 3, // 0,1,2 reserved in spirit
            open_files: HashMap::new(),
        }
    }

    pub fn getcwd(&self) -> &std::path::Path {
        &self.cwd_path
    }

    // Should this be public??!!
    pub fn resolve_path(&mut self, path: &str) -> std::io::Result<u64> {
        let mut inode = if path.starts_with('/') {
            1
        } else {
            self.cwd_inode
        };

        let p = if path.is_empty() { "." } else { path };

        for comp in std::path::Path::new(p).components() {
            use std::path::Component;
            match comp {
                Component::RootDir => {
                    inode = 1;
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    // for your hello FS, parent of "/" is "/", but use ".." lookup for generality
                    let entry = self.proto.lookup(inode, "..")?;
                    inode = entry.nodeid;
                }
                Component::Normal(name_os) => {
                    let name = name_os.to_string_lossy().to_string();
                    let entry = self.proto.lookup(inode, &name)?;
                    inode = entry.nodeid;
                }
                _ => {}
            }
        }

        Ok(inode)
    }

    pub fn chdir(&mut self, path: &str) -> std::io::Result<()> {
        let inode = self.resolve_path(path)?;
        let attr_out = self.proto.getattr(inode)?;
        let mode = attr_out.attr.mode;

        if (mode & libc::S_IFMT) != libc::S_IFDIR {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "not a directory",
            ));
        }

        if path.starts_with('/') {
            self.cwd_path = std::path::PathBuf::from(path);
        } else {
            self.cwd_path.push(path);
        }
        self.cwd_inode = inode;
        Ok(())
    }

    pub fn open(&mut self, path: &str, flags: u32) -> std::io::Result<Fd> {
        let inode = self.resolve_path(path)?;
        let out = self.proto.open(inode, flags)?;

        let fd = self.next_fd;
        self.next_fd += 1;

        self.open_files.insert(
            fd,
            OpenFile {
                inode,
                fh: out.fh,
                offset: 0,
                flags,
            },
        );

        Ok(fd)
    }

    pub fn read(&mut self, fd: Fd, size: u32) -> std::io::Result<Vec<u8>> {
        let of = self
            .open_files
            .get_mut(&fd)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "bad fd"))?;

        let data = self.proto.read(of.inode, of.fh, of.offset, size)?;
        of.offset += data.len() as u64;
        Ok(data)
    }

    pub fn close(&mut self, fd: Fd) -> std::io::Result<()> {
        if let Some(of) = self.open_files.remove(&fd) {
            self.proto.release(of.inode, of.fh)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "bad fd"))
        }
    }

    pub fn readdir(&mut self, path: &str) -> std::io::Result<Vec<DirEntryInfo>> {
        let dir_ino = self.resolve_path(path)?;
        let open = self.proto.opendir(dir_ino)?;
        let fh = open.fh;

        let mut offset = 0;
        let mut out = Vec::new();

        loop {
            let entries = self.proto.readdir(dir_ino, fh, offset, 4096)?;
            if entries.is_empty() {
                break;
            }

            for e in &entries {
                out.push(DirEntryInfo {
                    name: e.name.clone(),
                    inode: e.ino,
                    mode: 0, // you can fill this with getattr later
                });
            }

            offset = entries.last().unwrap().offset;
        }

        self.proto.releasedir(dir_ino, fh)?;
        Ok(out)
    }

    pub fn mkdir(&mut self, path: &str, mode: u32) -> std::io::Result<()> {
        use std::path::Path;
        let pb = Path::new(path);
        let parent = pb.parent().unwrap_or(Path::new("."));
        let name_os = pb
            .file_name()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "empty name"))?;
        let name = name_os.to_string_lossy().to_string();

        let parent_ino = self.resolve_path(parent.to_string_lossy().as_ref())?;
        let _entry = self.proto.mkdir(parent_ino, &name, mode)?;
        Ok(())
    }

    pub fn stat(&mut self, path: &str) -> std::io::Result<FileStat> {
        let inode = self.resolve_path(path)?;
        let attr_out = self.proto.getattr(inode)?;
        let a = attr_out.attr;

        Ok(FileStat {
            inode,
            size: a.size,
            mode: a.mode,
            nlink: a.nlink,
            uid: a.uid,
            gid: a.gid,

            blocks: a.blocks,
            blksize: a.blksize,
            rdev: a.rdev,

            atime: a.atime,
            atime_nsec: a.atimensec,

            mtime: a.mtime,
            mtime_nsec: a.mtimensec,

            ctime: a.ctime,
            ctime_nsec: a.ctimensec,
        })
    }
}
