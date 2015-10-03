# PTY [![Build Status](https://travis-ci.org/hibariya/pty-rs.svg?branch=master)](https://travis-ci.org/hibariya/pty-rs)

The `pty` crate provides `pty::fork()`. That makes a parent process fork with new pseudo-terminal (PTY).

This crate depends on followings:

* `libc` library
* POSIX environment

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]

pty = "0.1"
```

and this to your crate root:

```rust
extern crate pty;
```

### pty::fork()

This function returns `pty::Child`. It represents the child process and its PTY.

For example, the following code spawns `tty(1)` command by `pty::fork()` and outputs the result of the command.

```rust
extern crate libc;
extern crate pty;

use std::ffi::CString;
use std::io::Read;
use std::ptr;

fn main()
{
    match pty::fork() {
        Ok(child) => {
            if child.pid() == 0 {
                // Child process just exec `tty`
                let mut ptrs = [CString::new("tty").unwrap().as_ptr(), ptr::null()];

                unsafe { libc::execvp(*ptrs.as_ptr(), ptrs.as_mut_ptr()) };
            }
            else {
                // Read output via PTY master
                let mut output     = String::new();
                let mut pty_master = child.pty().unwrap();

                match pty_master.read_to_string(&mut output) {
                    Ok(_nread)  => println!("child tty is: {}", output.trim()),
                    Err(e)      => panic!("read error: {}", e)
                }

                child.wait();
            }
        },
        Err(e) => panic!("pty::fork error: {}", e)
    }
}
```

When run this, we get new PTY in the child process.

```
$ tty
/dev/pts/5
$ cargo run
    Running `target/debug/example`
child tty is: /dev/pts/8
```

## Contributing

1. Fork it ( https://github.com/hibariya/pty-rs/fork )
2. Create your feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create a new Pull Request

## License

Copyright (c) 2015 Hika Hibariya

Distributed under the [MIT License](LICENSE.txt).
