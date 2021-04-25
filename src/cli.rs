use clap::{crate_version, App, Arg, SubCommand};

/// Build the CLI instance
pub fn build_cli() -> App<'static, 'static> {
    App::new("systemd-boot-friend-rs")
        .version(crate_version!())
        .about("Systemd-Boot Kernel Version Selector")
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize systemd-boot and install the newest kernel"),
        )
        .subcommand(SubCommand::with_name("list").about("List available kernels"))
        .subcommand(
            SubCommand::with_name("install-kernel")
                .about("Install specific version of kernel")
                .arg(
                    Arg::with_name("target")
                        .help("Target kernel in the list or the number of the kernel")
                        .index(1),
                ),
        )
}
