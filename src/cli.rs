use clap::Parser;

/// Kernel Version Manager for systemd-boot
#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Opts {
    #[clap(subcommand)]
    pub subcommands: Option<SubCommands>,
}

#[derive(Parser, Debug)]
pub enum SubCommands {
    /// Initialize systemd-boot-friend
    Init,
    /// List all available kernels
    List,
    Install(Install),
    /// List all installed kernels
    ListInstalled,
    Remove(Remove),
    /// Install all kernels and update boot entries
    Update,
}

/// Install the kernel specified
#[derive(Parser, Debug)]
pub struct Install {
    pub target: Option<String>,
    /// force overwrite the entry config or not
    #[clap(long, short)]
    pub force: bool,
}

/// Remove the kernel specified
#[derive(Parser, Debug)]
pub struct Remove {
    pub target: Option<String>,
}
