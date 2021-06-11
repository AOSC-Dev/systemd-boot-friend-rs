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
    MakeConf(MakeConf),
    List(List),
    Install(Install),
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "init")]
/// Initialize systemd-boot-friend
pub struct Init {}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "mkconf")]
/// Create systemd-boot entry config
pub struct MakeConf {
    /// force rewrite the entry config or not
    #[argh(switch, short = 'f')]
    pub force: bool,
}

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
}
