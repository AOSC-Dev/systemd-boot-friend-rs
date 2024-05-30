use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(about, author, version, arg_required_else_help(true))]
pub struct Opts {
    #[command(subcommand)]
    pub subcommands: Option<SubCommands>,
}

#[derive(Subcommand, Debug)]
pub enum SubCommands {
    /// Initialize systemd-boot-friend
    #[command(display_order = 1)]
    Init,
    /// Install all kernels and update boot entries
    #[command(display_order = 2)]
    Update,
    /// Install the kernels specified
    #[command(display_order = 3)]
    InstallKernel {
        targets: Vec<String>,
        /// Force overwrite the entry config or not
        #[arg(long, short)]
        force: bool,
    },
    /// Remove the kernels specified
    #[command(display_order = 4)]
    RemoveKernel { targets: Vec<String> },
    /// Select kernels to install or remove
    #[command(display_order = 5)]
    Select,
    /// List all available kernels
    #[command(display_order = 6)]
    ListAvailable,
    /// List all installed kernels
    #[command(display_order = 7)]
    ListInstalled,
    /// Configure systemd-boot
    #[command(display_order = 8)]
    Config,
    /// Set the default kernel
    #[command(display_order = 9)]
    SetDefault { target: Option<String> },
    /// Set the boot menu timeout
    #[command(display_order = 10)]
    SetTimeout { timeout: Option<u32> },
}
