use clap::{AppSettings, Clap, crate_version, crate_authors};

/// Kernel Version Manager for systemd-boot
#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
#[clap(version = crate_version!(), author = crate_authors!())]
pub struct Opts {
    #[clap(subcommand)]
    pub subcommand: Option<SubCommand>
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    Init(Init),
    List(List),
    Install(Install),
    ListInstalled(ListInstalled),
    Remove(Remove),
    Update(Update),
}

/// Initialize systemd-boot-friend
#[derive(Clap, Debug)]
pub struct Init;

/// List all available kernels
#[derive(Clap, Debug)]
pub struct List;

/// Install the kernel specified
#[derive(Clap, Debug)]
pub struct Install {
    pub target: Option<String>,
    /// force overwrite the entry config or not
    #[clap(long, short)]
    pub force: bool,
}

/// List all installed kernels
#[derive(Clap, Debug)]
pub struct ListInstalled;

/// Remove the kernel specified
#[derive(Clap, Debug)]
pub struct Remove {
    pub target: Option<String>,
}

/// Install all kernels and update boot entries
#[derive(Clap, Debug)]
pub struct Update;