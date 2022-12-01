use std::fs::File;
use std::io;
use std::io::{BufWriter, Read, Write};
use std::ops::Deref;
use std::path::PathBuf;
use std::process::Child;
use std::sync::RwLock;

use nix::libc::pid_t;
use nix::sys::signal::{kill, Signal};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::Pid;

pub struct SharedProcess {
	path: PathBuf,
}

impl Default for SharedProcess {
	fn default() -> Self {
		Self::new("pid")
	}
}

impl SharedProcess {
	pub fn new(path: impl Into<PathBuf>) -> Self {
		Self { path: path.into() }
	}

	pub fn set(&self, no: i32) -> Option<()> {
		let mut file = File::create(&self.path).ok()?;
		write!(file, "{}", no).ok();
		file.flush().ok()
	}

	pub fn get(&self) -> Option<i32> {
		let mut file = File::open(&self.path).ok()?;
		let mut content = String::new();
		file.read_to_string(&mut content).ok()?;
		let pid = content.parse::<i32>().ok()?;
		kill(Pid::from_raw(pid), Signal::SIGCONT).ok()?;
		Some(pid)
	}
}

pub struct ChildHandler<Sin: Write, Sout: Read, Serr: Read> {
	stdin: RwLock<BufWriter<Sin>>,
	stdout: RwLock<Sout>,
	stderr: RwLock<Serr>,
}

pub enum WrappedChild<Sin: Write, Sout: Read, Serr: Read> {
	Owned(Child, ChildHandler<Sin, Sout, Serr>),
	Attached(Pid, ChildHandler<Sin, Sout, Serr>),
}

impl<Sin: Write, Sout: Read, Serr: Read> Deref for WrappedChild<Sin, Sout, Serr> {
	type Target = ChildHandler<Sin, Sout, Serr>;

	fn deref(&self) -> &Self::Target {
		match self {
			WrappedChild::Owned(_, io) => { io }
			WrappedChild::Attached(_, io) => { io }
		}
	}
}

impl WrappedChild<Box<dyn Write>, Box<dyn Read>, Box<dyn Read>> {
	pub fn new(mut child: Child) -> Option<Self> {
		let stdin = child.stdin.take()?;
		let stdout = child.stdout.take()?;
		let stderr = child.stderr.take()?;
		Some(WrappedChild::Owned(child, ChildHandler {
			stdin: RwLock::new(BufWriter::new(Box::new(stdin))),
			stdout: RwLock::new(Box::new(stdout)),
			stderr: RwLock::new(Box::new(stderr)),
		}))
	}
	pub fn attach(pid: Pid) -> Option<Self> {
		let stdin = File::create("stdin").ok()?;
		let stdout = File::open("stdout").ok()?;
		let stderr = File::open("stderr").ok()?;
		Some(WrappedChild::Attached(pid, ChildHandler {
			stdin: RwLock::new(BufWriter::new(Box::new(stdin))),
			stdout: RwLock::new(Box::new(stdout)),
			stderr: RwLock::new(Box::new(stderr)),
		}))
	}
}

impl<Sin: Write, Sout: Read, Serr: Read> WrappedChild<Sin, Sout, Serr> {
	pub fn write(&self, content: impl AsRef<[u8]>) -> io::Result<()> {
		let mut write = self.stdin.write().unwrap();
		write.write_all(content.as_ref())
	}

	pub fn flush(&self) -> io::Result<()> {
		let mut write = self.stdin.write().unwrap();
		write.flush()
	}

	pub fn read_stdout(&self, buf: &mut [u8]) -> io::Result<usize> {
		let mut write = self.stdout.write().unwrap();
		write.read(buf)
	}

	pub fn read_stderr(&self, buf: &mut [u8]) -> io::Result<usize> {
		let mut write = self.stderr.write().unwrap();
		write.read(buf)
	}

	pub fn pid(&self) -> Pid {
		match self {
			WrappedChild::Owned(child, _) => {
				Pid::from_raw(child.id() as pid_t)
			}
			WrappedChild::Attached(pid, _) => {
				*pid
			}
		}
	}

	pub fn wait(&self) -> nix::Result<WaitStatus> {
		waitpid(self.pid(), None)
	}
}