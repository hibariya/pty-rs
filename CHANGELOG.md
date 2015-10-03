### 0.1.5

Remove unnecessary `Copy` trait.

* API Change: [#3](https://github.com/hibariya/pty-rs/pull/3)
  * Mark `Child#pty` as private, add public `Child#pty()`.
  * Remove `Copy` trait from `Child` and `ChildPTY`.
  * Remove `ChildPTY#fd()`, impl `AsRawFd` for `ChildPTY`.

### 0.1.4

* API Change: [#2](https://github.com/hibariya/pty-rs/pull/2) Make `pty::fork()` return a single value.

### 0.1.3

* Support stable rust
