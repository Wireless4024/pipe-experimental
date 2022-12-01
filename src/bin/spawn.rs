use std::{env, process};
use std::env::args_os;
use std::fs::{File, OpenOptions};
use std::io::{Read, stdin};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use nix::sys::signal::{kill, Signal};
use nix::sys::wait::waitpid;
use nix::unistd::{fork, ForkResult, Pid};

use pipe_experimental::SharedProcess;

pub fn main() {
	if env::var_os("RUN_AS_CHILD").is_some() {
		let mut stdin = stdin().lock();
		let mut buf = vec![0; 4096];
		loop {
			let len = stdin.read(&mut buf).unwrap();
			println!("stdin: {}", String::from_utf8_lossy(&buf[..len]));
		}
	} else if env::var_os("HOLD").is_some() {
		let _stdin = File::create("stdin").unwrap();
		let _stdout = File::open("stdout").unwrap();
		let _stderr = File::open("stderr").unwrap();
		// pause itself this will release when receive `SIGCONT`
		kill(Pid::from_raw(process::id() as _), Signal::SIGSTOP).expect("PAUSE");
		sleep(Duration::from_millis(500));
	} else {
		match unsafe { fork() }.expect("Can't fork") {
			ForkResult::Parent { child } => {
				println!("waiting");
				waitpid(Some(child), None).unwrap();
			}
			ForkResult::Child => {
				nix::unistd::setsid().expect("Create new session");
				let this = args_os().next().unwrap();
				let guard = Command::new(&this)
					.stdin(Stdio::null())
					.stdout(Stdio::null())
					.stderr(Stdio::null())
					.env("HOLD", "YE")
					.spawn()
					.unwrap();

				let in_file = Stdio::from(OpenOptions::new().read(true).open("stdin").unwrap());
				let out_file = Stdio::from(OpenOptions::new().append(true).open("stdout").unwrap());
				let err_file = Stdio::from(OpenOptions::new().append(true).open("stderr").unwrap());
				SharedProcess::new("guard.id").set(guard.id() as _);
				println!("spawning {:?} as child", this);
				let mut _child = Command::new(this)
					.stdin(in_file)
					.stdout(out_file)
					.stderr(err_file)
					.env("RUN_AS_CHILD", "YE")
					.spawn()
					.unwrap();
				let pid = _child.id() as i32;
				let p = SharedProcess::default();
				p.set(pid);
				println!("saved pid");
				//	_child.wait().unwrap();
			}
		}
	}
}