use std::io::{self, Write};

use crate::transport::common::FuseTransport;
use crate::virtiofs::VirtioFsImpl;
use crate::virtiofs::structs::FileStat;

pub struct FuseShell<T: FuseTransport> {
    vfs: VirtioFsImpl<T>,
}

impl<T: FuseTransport> FuseShell<T> {
    pub fn new(vfs: VirtioFsImpl<T>) -> Self {
        Self { vfs }
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            print!("fuse:{}> ", self.vfs.getcwd().display());
            std::io::stdout().flush()?;

            let mut line = String::new();
            if io::stdin().read_line(&mut line)? == 0 {
                break;
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let mut parts = line.split_whitespace();
            let cmd = parts.next().unwrap();
            let args: Vec<&str> = parts.collect();

            match cmd {
                "ls" => {
                    /*if args.len() == 1 && args[0] == "-l" {
                        if let Err(e) = self.cmd_ls_l() {
                            eprintln!("ls -l: {}", e);
                        }
                    } else*/
                    {
                        let path = if args.is_empty() { "." } else { args[0] };
                        if let Err(e) = self.cmd_ls(path) {
                            eprintln!("ls: {}", e);
                        }
                    }
                }

                "mkdir" => {
                    if args.is_empty() {
                        println!("Usage: mkdir <path>");
                        continue;
                    }
                    if let Err(e) = self.vfs.mkdir(args[0], libc::S_IFDIR as u32 | 0o755) {
                        eprintln!("mkdir: {}", e);
                    }
                }

                "stat" => {
                    if args.is_empty() {
                        println!("Usage: stat <path>");
                        continue;
                    }
                    let path = args[0];
                    match self.vfs.stat(path) {
                        Ok(st) => {
                            self.cmd_stat(st);
                        }

                        Err(e) => {
                            eprintln!("stat: {}", e);
                        }
                    }
                }

                "cat" => {
                    if args.is_empty() {
                        println!("Usage: cat <path>");
                        continue;
                    }
                    if let Err(e) = self.cmd_cat(args[0]) {
                        eprintln!("cat: {}", e);
                    }
                }

                "cd" => {
                    if args.is_empty() {
                        println!("Usage: cd <path>");
                        continue;
                    }
                    if let Err(e) = self.vfs.chdir(args[0]) {
                        eprintln!("cd: {}", e);
                    }
                }

                "pwd" => {
                    println!("{}", self.vfs.getcwd().display());
                }

                "exit" | "quit" => break,

                _ => println!("unknown command: {}", cmd),
            }
        }
        Ok(())
    }

    fn cmd_stat(&self, st: FileStat) -> () {
        use std::time::{Duration, UNIX_EPOCH};

        // File type
        let ftype = match st.mode & libc::S_IFMT {
            libc::S_IFREG => "regular file",
            libc::S_IFDIR => "directory",
            libc::S_IFLNK => "symbolic link",
            libc::S_IFCHR => "character device",
            libc::S_IFBLK => "block device",
            libc::S_IFIFO => "FIFO/pipe",
            libc::S_IFSOCK => "socket",
            _ => "unknown",
        };

        // Permissions rwxr-xr-x
        fn mode_to_string(mode: u32) -> String {
            let mut s = String::new();
            let perms = [
                (libc::S_IRUSR, 'r'),
                (libc::S_IWUSR, 'w'),
                (libc::S_IXUSR, 'x'),
                (libc::S_IRGRP, 'r'),
                (libc::S_IWGRP, 'w'),
                (libc::S_IXGRP, 'x'),
                (libc::S_IROTH, 'r'),
                (libc::S_IWOTH, 'w'),
                (libc::S_IXOTH, 'x'),
            ];
            for &(bit, ch) in &perms {
                s.push(if mode & bit != 0 { ch } else { '-' });
            }
            s
        }

        let perm_string = mode_to_string(st.mode);

        // Timestamp formatter
        fn fmt_time(sec: u64, nsec: u32) -> String {
            let ts = UNIX_EPOCH + Duration::new(sec, nsec);
            let dt: chrono::DateTime<chrono::Local> = ts.into();
            dt.format("%Y-%m-%d %H:%M:%S.%f").to_string()
        }

        println!(
            "  Size: {:<10} Blocks: {:<10} IO Block: {}",
            st.size, st.blocks, st.blksize
        );
        println!(
            "Device: {} Inode: {}  Links: {}",
            st.rdev, st.inode, st.nlink
        );
        println!(
            "Access: ({:o}/{})  Uid: {}  Gid: {}",
            st.mode & 0o7777,
            perm_string,
            st.uid,
            st.gid
        );
        println!("Access: {}", fmt_time(st.atime, st.atime_nsec));
        println!("Modify: {}", fmt_time(st.mtime, st.mtime_nsec));
        println!("Change: {}", fmt_time(st.ctime, st.ctime_nsec));
        println!("Type: {}", ftype);
    }

    fn cmd_ls(&mut self, path: &str) -> std::io::Result<()> {
        let entries = self.vfs.readdir(path)?;
        for e in entries {
            println!("{}", e.name);
        }
        Ok(())
    }

    fn cmd_cat(&mut self, path: &str) -> std::io::Result<()> {
        let fd = self.vfs.open(path, libc::O_RDONLY as u32)?;
        let data = self.vfs.read(fd, 64 * 1024)?; // read whole file; your FS is tiny
        self.vfs.close(fd)?;
        print!("{}", String::from_utf8_lossy(&data));
        Ok(())
    }
}
