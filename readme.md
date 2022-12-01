# Pipe Experimental

This is experiment about linux pipe to make child process
able to reattach by parent process again.

The idea is redirect stdio of process into named pipe and hold it,
by default it use anonymous-pipe, but it will close when parent exit.
so this experiment use named pipe and dangling file handler to keep
stdio pipe open.

It's work right? yes!

# Create pipe

```shell
mkfifo stdin
mkfifo stdout
mkfifo stderr
```

# Create orphan child

```shell
cargo run --bin spawn
```

Here you can manipulate stdio of process by

```shell
# send message to stdin
echo hello > stdin

# read from stdout
echo It\'s safe to ctrl-c here, it\'s wont kill or close actual pipe
cat stdout
```

Or you can use existing rust script to do it

```shell
cargo run --bin attach
```

# Kill child when not read

**Note:** Don't forgot to kill running process! It will be a zombie in your system.

```shell
cargo run --bin release
```