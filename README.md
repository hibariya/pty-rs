# PTY

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

### `pty::fork() -> io::Result<(pty::Child, pty::Master)>`

This function returns two values. `pty::Child` represents a child process. `pty::Master` represents master of a PTY.

For example, the following code spawns `tty` command by `pty::fork()`.

```rust
extern crate libc;
extern crate pty;

use std::io::Read;

fn main()
{
    match pty::fork() {
        Ok((child, mut pty_master)) => {
            if child.pid() == 0 {
                let mut ptrs: Vec<*const libc::c_char> = Vec::with_capacity(1);

                ptrs.push(std::ffi::CString::new("tty").unwrap().as_ptr());
                ptrs.push(std::ptr::null());

                unsafe { libc::execvp(*ptrs.as_ptr(), ptrs.as_mut_ptr()) };
            }
            else {
                let mut string = String::new();

                match pty_master.read_to_string(&mut string) {
                    Ok(_nread)  => {
                        println!("child tty is: {}", string.trim());
                    },
                    Err(e) => panic!("read error: {}", e)
                }

                child.wait();
                pty_master.close();
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
