#[macro_export]
macro_rules! println_with_prefix {
    ($($arg:tt)+) => {
        print!("\u{001b}[1m[systemd-boot-friend]\u{001b}[0m ");
        println!($($arg)+);
    };
}

#[macro_export]
macro_rules! yield_into {
    { ( $x:ident ) = $v:expr, $e:expr, $f:ident } => {
        $x = $v.next().ok_or_else(|| anyhow!("{}: {}", $e, $f))?;
    };
    { ( $x:ident, $($y:ident),+ ) = $v:expr, $e:expr, $f:ident } => {
        $x = $v.next().ok_or_else(|| anyhow!("{}: {}", $e, $f))?;
        yield_into!(($($y),+) = $v, $e, $f);
    }
}
