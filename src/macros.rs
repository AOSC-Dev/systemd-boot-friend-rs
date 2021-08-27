#[macro_export]
macro_rules! println_with_prefix {
    ($($arg:tt)+) => {
        eprint!("\u{001b}[1m[systemd-boot-friend]\u{001b}[0m ");
        eprintln!($($arg)+);
    };
}

#[macro_export]
macro_rules! println_with_prefix_and_fl {
    ($message_id:literal) => {
        println_with_prefix!("{}", fl!($message_id));
    };

    ($message_id:literal, $($args:expr), *) => {
        println_with_prefix!("{}", fl!($message_id, $($args), *));
    };
}