use clap::Shell;
use std::env;

include!("src/cli.rs");

const GENERATED_COMPLETIONS: &[Shell] = &[Shell::Bash, Shell::Zsh, Shell::Fish];

fn main() {
    // generate completions on demand
    if env::var("GEN_COMPLETIONS").is_ok() {
        let mut app = build_cli();
        for shell in GENERATED_COMPLETIONS {
            app.gen_completions("systemd-boot-friend", *shell, "completions");
        }
    }
}
