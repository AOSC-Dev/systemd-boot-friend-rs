#[macro_export]
macro_rules! println_with_prefix {
    ($($arg:tt)+) => {
        eprint!("\u{001b}[1m[systemd-boot-friend]\u{001b}[0m ");
        eprintln!($($arg)+);
    };
}

#[macro_export]
macro_rules! print_block_with_fl {
    ($message_id:literal) => {
        eprintln!("\n{}\n", fl!($message_id))
    };

    ($message_id:literal, $($args:expr), *) => {
        eprintln!("\n{}\n", fl!($message_id, $($args), *))
    }
}

#[macro_export]
macro_rules! println_with_prefix_and_fl {
    ($message_id:literal) => {
        for line in fl!($message_id).lines() {
            println_with_prefix!("{}", line);
        }
    };

    ($message_id:literal, $($args:expr), *) => {
        for line in fl!($message_id, $($args), *).lines() {
            println_with_prefix!("{}", line);
        }
    };
}
