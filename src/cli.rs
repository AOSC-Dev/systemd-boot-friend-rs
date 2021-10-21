use clap::{Parser, crate_version, crate_authors};

/// Kernel Version Manager for systemd-boot
#[derive(Parser, Debug)]
#[clap(version = crate_version!(), author = crate_authors!())]
pub struct Opts {
    #[clap(subcommand)]
    pub subcommand: Option<SubCommand>
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    Init(Init),
    List(List),
    Install(Install),
    ListInstalled(ListInstalled),
    Remove(Remove),
    Update(Update),
}

/// Initialize systemd-boot-friend
#[derive(Parser, Debug)]
pub struct Init;

/// List all available kernels
#[derive(Parser, Debug)]
pub struct List;

/// Install the kernel specified
#[derive(Parser, Debug)]
pub struct Install {
    pub target: Option<String>,
    /// force overwrite the entry config or not
    #[clap(long, short)]
    pub force: bool,
}

/// List all installed kernels
#[derive(Parser, Debug)]
pub struct ListInstalled;

/// Remove the kernel specified
#[derive(Parser, Debug)]
pub struct Remove {
    pub target: Option<String>,
}

/// Install all kernels and update boot entries
#[derive(Parser, Debug)]
pub struct Update;