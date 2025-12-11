use std::io::{self, Write};
use std::path::PathBuf;

use crate::protocol::FuseProtocol;

pub struct FuseShell {
    proto: FuseProtocol,
    cwd_inode: u64,
    cwd_path: PathBuf,
}

impl FuseShell {
    pub fn new(proto: FuseProtocol) -> io::Result<Self> {
        Ok(Self {
            proto,
            cwd_inode: 1,
            cwd_path: PathBuf::from("/"),
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            print!("fuse:{}> ", self.cwd_path.display());
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
                "ls" => self.cmd_ls()?,
                "stat" => {
                    if args.is_empty() { println!("Usage: stat <name>"); continue; }
                    self.cmd_stat(args[0])?;
                }
                "cat" => {
                    if args.is_empty() { println!("Usage: cat <name>"); continue; }
                    self.cmd_cat(args[0])?;
                }
                "cd" => {
                    if args.is_empty() { println!("Usage: cd <name>"); continue; }
                    self.cmd_cd(args[0])?;
                }
                "pwd" => println!("{}", self.cwd_path.display()),
                "exit" | "quit" => break,
                _ => println!("unknown command: {}", cmd),
            }
        }
        Ok(())
    }

    fn lookup_name(&mut self, name: &str) -> io::Result<u64> {
        let entry = self.proto.lookup(self.cwd_inode, name)?;
        Ok(entry.nodeid)
    }

    fn cmd_ls(&mut self) -> io::Result<()> {
        println!("ls is not implemented yet (needs FUSE_READDIR).");
        Ok(())
    }

    fn cmd_stat(&mut self, name: &str) -> io::Result<()> {
        let inode = self.lookup_name(name)?;
        let attr = self.proto.getattr(inode)?;

        println!("inode={} size={} mode={:o}",
            attr.attr.ino, attr.attr.size, attr.attr.mode);

        Ok(())
    }

    fn cmd_cat(&mut self, name: &str) -> io::Result<()> {
        let inode = self.lookup_name(name)?;
        let open_out = self.proto.open(inode, libc::O_RDONLY as u32)?;

        let mut offset = 0u64;
        loop {
            let data = self.proto.read(inode, open_out.fh, offset, 4096)?;
            if data.is_empty() {
                break;
            }
            print!("{}", String::from_utf8_lossy(&data));
            offset += data.len() as u64;
        }

        self.proto.release(inode, open_out.fh)?;
        Ok(())
    }

    fn cmd_cd(&mut self, name: &str) -> io::Result<()> {
        let inode = self.lookup_name(name)?;
        let attr = self.proto.getattr(inode)?;

        const S_IFDIR: u32 = 0o040000;

        if attr.attr.mode & S_IFDIR == 0 {
            println!("{} is not a directory", name);
            return Ok(());
        }

        if name == ".." {
            self.cwd_path.pop();
        } else if name != "." {
            self.cwd_path.push(name);
        }

        self.cwd_inode = inode;
        Ok(())
    }
}
