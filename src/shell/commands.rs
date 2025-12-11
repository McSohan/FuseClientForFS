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
                    let long = args.contains(&"-l");
                    let path = args
                        .iter()
                        .find(|a| !a.starts_with('-'))
                        .copied()
                        .unwrap_or(".");

                    let res = if long {
                        self.cmd_ls_long(path)
                    } else {
                        self.cmd_ls_short(path)
                    };

                    if let Err(e) = res {
                        eprintln!("ls: {}", e);
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
                        Ok(st) => self.cmd_stat(st),
                        Err(e) => eprintln!("stat: {}", e),
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

    /* ---------------------------------------------------------------------
    stat formatting
    --------------------------------------------------------------------- */
    fn cmd_stat(&self, st: FileStat) -> () {
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

        fn fmt_time(sec: u64, nsec: u32) -> String {
            use chrono::{DateTime, Local, LocalResult, TimeZone};

            let dt: DateTime<Local> = match Local.timestamp_opt(sec as i64, nsec) {
                LocalResult::Single(t) => t,
                _ => Local.timestamp_opt(0, 0).unwrap(),
            };

            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        }

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

        let perm_string = mode_to_string(st.mode);

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

    /* ---------------------------------------------------------------------
    ls (short): behaves like ls [path]
    --------------------------------------------------------------------- */
    fn cmd_ls_short(&mut self, path: &str) -> std::io::Result<()> {
        // First, stat the path to see if it is a file or directory.
        let st = self.vfs.stat(path)?;

        if (st.mode & libc::S_IFMT) == libc::S_IFDIR {
            // Directory -> list contents.
            let entries = self.vfs.readdir(path)?;
            for e in entries {
                println!("{}", e.name);
            }
        } else {
            // Regular file (or other non-dir) -> just print the path itself.
            println!("{}", path);
        }

        Ok(())
    }

    /* ---------------------------------------------------------------------
    ls -l
    --------------------------------------------------------------------- */
    fn cmd_ls_long(&mut self, path: &str) -> std::io::Result<()> {
        // Determine if path is file or directory.
        let st = self.vfs.stat(path)?;

        if (st.mode & libc::S_IFMT) != libc::S_IFDIR {
            // It's a single file: print one long line.
            let mode_str = Self::format_mode(st.mode);
            let time = Self::format_time(st.mtime);

            println!(
                "{} {:>2} {:>4} {:>4} {:>8} {} {}",
                mode_str, st.nlink, st.uid, st.gid, st.size, time, path
            );
            return Ok(());
        }

        // Directory: list entries.
        let entries = self.vfs.readdir(path)?;

        for e in entries {
            // Build a path string for stat() that respects ".", "/", and subdirs.
            let full_path = if path == "." {
                e.name.clone()
            } else if path == "/" {
                format!("/{}", e.name)
            } else {
                format!("{}/{}", path.trim_end_matches('/'), e.name)
            };

            match self.vfs.stat(&full_path) {
                Ok(st) => {
                    let mode_str = Self::format_mode(st.mode);
                    let time = Self::format_time(st.mtime);

                    println!(
                        "{} {:>2} {:>4} {:>4} {:>8} {} {}",
                        mode_str, st.nlink, st.uid, st.gid, st.size, time, e.name
                    );
                }
                Err(err) => {
                    // Do not abort the whole listing just because one entry fails.
                    eprintln!("ls -l: {}: {}", full_path, err);
                }
            }
        }

        Ok(())
    }

    /* --- helpers for ls -l --- */

    fn format_mode(mode: u32) -> String {
        let ftype = match mode & libc::S_IFMT {
            libc::S_IFDIR => 'd',
            libc::S_IFLNK => 'l',
            libc::S_IFREG => '-',
            libc::S_IFCHR => 'c',
            libc::S_IFBLK => 'b',
            libc::S_IFIFO => 'p',
            libc::S_IFSOCK => 's',
            _ => '?',
        };

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

        let mut s = String::with_capacity(10);
        s.push(ftype);

        for (bit, ch) in perms {
            s.push(if mode & bit != 0 { ch } else { '-' });
        }

        s
    }

    fn format_time(secs: u64) -> String {
        use chrono::{DateTime, Local, LocalResult, TimeZone};

        let dt: DateTime<Local> = match Local.timestamp_opt(secs as i64, 0) {
            LocalResult::Single(t) => t,
            _ => Local.timestamp_opt(0, 0).unwrap(),
        };

        dt.format("%b %d %H:%M").to_string()
    }

    /* ---------------------------------------------------------------------
    cat
    --------------------------------------------------------------------- */
    fn cmd_cat(&mut self, path: &str) -> std::io::Result<()> {
        let fd = self.vfs.open(path, libc::O_RDONLY as u32)?;
        let data = self.vfs.read(fd, 64 * 1024)?;
        self.vfs.close(fd)?;
        print!("{}", String::from_utf8_lossy(&data));
        Ok(())
    }
}
