#[macro_export]
macro_rules! println_with_prefix {
    ($($arg:tt)+) => {
        eprint!("\u{001b}[1m[systemd-boot-friend]\u{001b}[0m ");
        eprintln!($($arg)+);
    };
}
