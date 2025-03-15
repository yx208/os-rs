# OS-Rust

## Compiler option

### Linux

```shell
cargo rustc -- -C link-arg=-nostartfiles
```

### Windows

```shell
cargo rustc -- -C link-args="/ENTRY:_start /SUBSYSTEM:console"
```

### macOS

```shell
cargo rustc -- -C link-args="-e __start -static -nostartfiles"
```