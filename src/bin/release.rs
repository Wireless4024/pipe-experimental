use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

use pipe_experimental::SharedProcess;


macro_rules! resume {
    ($pid:expr) => {
		if let Some(pid) = $pid {
			kill(Pid::from_raw(pid as _), Signal::SIGCONT).expect("resume process");
			kill(Pid::from_raw(pid as _), Signal::SIGINT).expect("interrupt process");
		}
    };
}
fn main() {
	resume!(SharedProcess::new("guard.id").get());
}