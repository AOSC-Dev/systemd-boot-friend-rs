#[macro_export]
macro_rules! println_with_prefix {
    ($($arg:tt)+) => {
        print!("\u{001b}[1m[systemd-boot-friend]\u{001b}[0m ");
        println!($($arg)+);
    };
}

#[macro_export]
macro_rules! yield_into {
    { ( $x:ident ) = $v:expr, $e:expr } => {
        $x = $v.next().ok_or_else(|| anyhow!("{}", $e))?;
    };
    { ( $x:ident, $($y:ident),+ ) = $v:expr, $e:expr } => {
        $x = $v.next().ok_or_else(|| anyhow!("{}", $e))?;
        yield_into!(($($y),+) = $v, $e);
    }
}
