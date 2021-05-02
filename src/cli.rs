use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Systemd-Boot Kernel Version Selector
pub struct Interface {
    #[argh(subcommand)]
    pub nested: Option<SubCommandEnum>,
    #[argh(switch)]
    /// show the version of systemd-boot-friend
    pub version: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum SubCommandEnum {
    Init(Init),
    List(List),
    InstallKernel(InstallKernel),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "init")]
/// Initialize systemd-boot-friend
pub struct Init {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "list")]
/// List all available kernels
pub struct List {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "install-kernel")]
/// Install a specific kernel
pub struct InstallKernel {
    #[argh(positional)]
    pub target: Option<String>,
}
