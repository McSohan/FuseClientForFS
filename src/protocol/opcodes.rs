//! Fuse opcodes as defined in fs/fuse/fuse_i.h

// ========== File system operations ==========

pub const FUSE_LOOKUP: u32              = 1;
pub const FUSE_FORGET: u32              = 2;   // no reply
pub const FUSE_GETATTR: u32             = 3;
pub const FUSE_SETATTR: u32             = 4;
pub const FUSE_READLINK: u32            = 5;
pub const FUSE_SYMLINK: u32             = 6;
pub const FUSE_MKNOD: u32               = 8;
pub const FUSE_MKDIR: u32               = 9;
pub const FUSE_UNLINK: u32              = 10;
pub const FUSE_RMDIR: u32               = 11;
pub const FUSE_RENAME: u32              = 12;
pub const FUSE_LINK: u32                = 13;

// ========== File operations ==========

pub const FUSE_OPEN: u32                = 14;
pub const FUSE_READ: u32                = 15;
pub const FUSE_WRITE: u32               = 16;
pub const FUSE_STATFS: u32              = 17;
pub const FUSE_RELEASE: u32             = 18;
pub const FUSE_FSYNC: u32               = 20;

// ========== Directory operations ==========

pub const FUSE_SETXATTR: u32            = 21;
pub const FUSE_GETXATTR: u32            = 22;
pub const FUSE_LISTXATTR: u32           = 23;
pub const FUSE_REMOVEXATTR: u32         = 24;

pub const FUSE_FLUSH: u32               = 25;

// ========== Session / mount operations ==========

pub const FUSE_INIT: u32                = 26;
pub const FUSE_OPENDIR: u32             = 27;
pub const FUSE_READDIR: u32             = 28;
pub const FUSE_RELEASEDIR: u32          = 29;
pub const FUSE_FSYNCDIR: u32            = 30;

pub const FUSE_GETLK: u32               = 31;
pub const FUSE_SETLK: u32               = 32;
pub const FUSE_SETLKW: u32              = 33;

pub const FUSE_ACCESS: u32              = 34;
pub const FUSE_CREATE: u32              = 35;

// ========== Interrupt + IOCTL ==========

pub const FUSE_INTERRUPT: u32           = 36;

pub const FUSE_BMAP: u32                = 37;

pub const FUSE_DESTROY: u32             = 38;

// ========== FS notifications & extended ops ==========

pub const FUSE_IOCTL: u32               = 39;

pub const FUSE_POLL: u32                = 40;   // kernel >= 3.11

pub const FUSE_NOTIFY_REPLY: u32        = 41;

// ========== Batch & writeback ops ==========

pub const FUSE_BATCH_FORGET: u32        = 42;

// ========== FUSE_READDIRPLUS ==========

pub const FUSE_READDIRPLUS: u32         = 43;   // kernel >= 3.13

pub const FUSE_RENAME2: u32             = 44;   // flags-enabled rename

// ========== Symlink + contact ==========

pub const FUSE_LSEEK: u32               = 45;

// ========== Copy file range ==========

pub const FUSE_COPY_FILE_RANGE: u32     = 46;   // kernel >= 4.5

// ========== Additional ops for virtiofs ==========

// FALLTHROUGH_FLAGS was removed from kernel, but opcode reserved
pub const FUSE_SETUPMAPPING: u32        = 47;   // virtio-fs / dax
pub const FUSE_REMOVEMAPPING: u32       = 48;   // virtio-fs / dax

// ========== Reserved opcodes (placeholders) ==========

// 49–63 reserved for future expansion
