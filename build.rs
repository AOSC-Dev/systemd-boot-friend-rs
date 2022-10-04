use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use std::{env, fs, io::Result};

include!("src/cli.rs");

const ROOT: &str = "completions";
const APP: &str = "sbf";
const GENERATED_COMPLETIONS: &[Shell] = &[Shell::Bash, Shell::Zsh, Shell::Fish];

fn generate_completions() -> Result<()> {
    fs::create_dir_all(ROOT)?;
    let mut app = Opts::command();
    for shell in GENERATED_COMPLETIONS {
        generate_to(*shell, &mut app, APP, ROOT)?;
    }

    Ok(())
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-env-changed=SBF_GEN_COMPLETIONS");
    if env::var("SBF_GEN_COMPLETIONS").is_ok() {
        generate_completions()?;
    }

    Ok(())
}
