use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, author, version, arg_required_else_help(true))]
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
    /// Configure systemd-boot
    #[clap(display_order = 7)]
    Config,
    #[clap(display_order = 8)]
    SetDefault(SetDefault),
    #[clap(display_order = 9)]
    SetTimeout(SetTimeout),
}

/// Install the kernels specified
#[derive(Parser, Debug)]
pub struct Install {
    pub targets: Vec<String>,
    /// Force overwrite the entry config or not
    #[clap(long, short)]
    pub force: bool,
}

/// Remove the kernels specified
#[derive(Parser, Debug)]
pub struct Remove {
    pub targets: Vec<String>,
}

/// Set the default kernel
#[derive(Parser, Debug)]
pub struct SetDefault {
    pub target: Option<String>,
}

/// Set the boot menu timeout
#[derive(Parser, Debug)]
pub struct SetTimeout {
    pub timeout: Option<u32>,
}
