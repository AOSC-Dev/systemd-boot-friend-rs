use clap::IntoApp;
use clap_generate::{
    generate,
    generators::{Bash, Fish, Zsh},
    Generator,
};
use std::{env, fs, io::Result};

include!("src/cli.rs");

const ROOT: &str = "completions";
const APP: &str = "systemd-boot-friend";

macro_rules! generate_shell_completions {
    ($app:ident, $shell:ident, $($shells:ident),+) => {
        generate_shell_completions!($app, $shell);
        generate_shell_completions!($app, $($shells),+);
    };

    ($app:ident, $shell:ident) => {
        generate::<$shell, _>(
            &mut $app,
            APP,
            &mut fs::File::create(format!("{}/{}", ROOT, $shell::file_name(APP)))?,
        );
    };
}

fn generate_completions() -> Result<()> {
    fs::create_dir_all(ROOT)?;
    let mut app = Opts::into_app();
    generate_shell_completions!(app, Bash, Zsh, Fish);

    Ok(())
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-env-changed=SBF_GEN_COMPLETIONS");
    if env::var("SBF_GEN_COMPLETIONS").is_ok() {
        generate_completions()?;
    }

    Ok(())
}
