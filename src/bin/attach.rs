use std::process::exit;

use nix::unistd::Pid;

use pipe_experimental::{SharedProcess, WrappedChild};

fn main() {
	let pid = match SharedProcess::default().get() {
		Some(id) => { id }
		None => {
			eprintln!("Process doesn't existed, please run `spawn` first");
			exit(1);
		}
	};
	let child = WrappedChild::attach(Pid::from_raw(pid as _)).expect("Attach");
	println!("writing `Hello` to child");
	child.write("Hello").unwrap();
	child.flush().unwrap();
	let mut out = vec![0; 4096];
	println!("waiting response");
	let  len= child.read_stdout(&mut out).unwrap();
	//child.read_stderr(&mut err).unwrap();
	println!("stdout:{}", String::from_utf8_lossy(&out[..len]));
}