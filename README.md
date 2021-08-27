# Your systemd-boot's best friend ever (Implemented in Rust)

A kernel version manager for systemd-boot and AOSC OS

## Usage

First initialize friend and systemd-boot, this will also
install the newest kernel to the specific path for systemd-boot.

```bash
systemd-boot-friend init
```

You can also manually choose the kernel you would like to install.

```bash
systemd-boot-friend
```

Subcommands are also supported, you may look up for them by
executing the following command.

```bash
systemd-boot-friend --help
```

For further information, visit https://wiki.aosc.io/software/systemd-boot-friend/

## Installation

```bash
cargo build --release
install -Dm755 target/release/systemd-boot-friend-rs /usr/local/bin/systemd-boot-friend
```

## Dependencies

Building:

- Rust w/ Cargo
- C compiler
- make (when GCC LTO is used, not needed for Clang)

Runtime:

- Systemd
