use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// Kernel Version Manager for systemd-boot
pub struct Interface {
    #[argh(subcommand)]
    pub nested: Option<SubCommandEnum>,
    #[argh(switch, short = 'V')]
    /// show the version of systemd-boot-friend
    pub version: bool,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum SubCommandEnum {
    Init(Init),
    List(List),
    Install(Install),
    ListInstalled(ListInstalled),
    Remove(Remove),
    Update(Update),
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "init")]
/// Initialize systemd-boot-friend
pub struct Init {}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "list")]
/// List all available kernels
pub struct List {}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "install")]
/// Install the kernel specified
pub struct Install {
    #[argh(positional)]
    pub target: Option<String>,
    /// force overwrite the entry config or not
    #[argh(switch, short = 'f')]
    pub force: bool,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "list-installed")]
/// List all installed kernels
pub struct ListInstalled {}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "remove")]
/// Remove the kernel specified
pub struct Remove {
    #[argh(positional)]
    pub target: Option<String>,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "update")]
/// Install all kernels and update boot entries
pub struct Update {}