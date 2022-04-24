# Your systemd-boot's best friend ever (hopefully)

A kernel version manager for systemd-boot

## Usage

First initialize friend and systemd-boot, this will also
install the newest kernel to the specific path for systemd-boot.

```bash
sbf init
```

You can also manually select the kernel(s) you would like to register as boot 
entry(s).

```bash
sbf install-kernel
```

Subcommands are also supported, you may look up for them by
executing the following command.

```bash
sbf --help
```

For further information, visit https://wiki.aosc.io/software/systemd-boot-friend/

## Installation

```bash
cargo build --release
install -Dm755 target/release/systemd-boot-friend /usr/local/bin/systemd-boot-friend
PREFIX=/usr/local ./install-assets.sh
```

Or from crates.io

```bash
cargo install systemd-boot-friend-rs
```

## Dependencies

Building:

- Rust w/ Cargo
- C compiler
- make (when GCC LTO is used, not needed for Clang)

Runtime:

- Systemd
