use clap::{AppSettings, Parser};

#[derive(Parser, Debug)]
#[clap(about, author, version, setting = AppSettings::ArgRequiredElseHelp)]
pub struct Opts {
    #[clap(subcommand)]
    pub subcommands: Option<SubCommands>,
}

#[derive(Parser, Debug)]
pub enum SubCommands {
    /// Initialize systemd-boot-friend
    #[clap(display_order = 1)]
    Init,
    /// Install all kernels and update boot entries
    #[clap(display_order = 2)]
    Update,
    #[clap(display_order = 3)]
    InstallKernel(Install),
    #[clap(display_order = 4)]
    RemoveKernel(Remove),
    /// List all available kernels
    #[clap(display_order = 5)]
    ListAvailable,
    /// List all installed kernels
    #[clap(display_order = 6)]
    ListInstalled,
}

/// Install the kernel specified
#[derive(Parser, Debug)]
pub struct Install {
    pub target: Option<String>,
    /// Force overwrite the entry config or not
    #[clap(long, short)]
    pub force: bool,
}

/// Remove the kernel specified
#[derive(Parser, Debug)]
pub struct Remove {
    pub target: Option<String>,
}
