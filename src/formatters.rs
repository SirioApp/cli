use ethers::types::U256;
use ethers::utils::format_units;

fn supports_color() -> bool {
    std::env::var_os("NO_COLOR").is_none()
        && std::env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
}

fn paint(text: impl AsRef<str>, code: &str) -> String {
    let t = text.as_ref();
    if supports_color() {
        format!("\x1b[{code}m{t}\x1b[0m")
    } else {
        t.to_string()
    }
}

pub fn header(text: impl AsRef<str>) -> String {
    paint(text, "1;36")
}

pub fn key(text: impl AsRef<str>) -> String {
    paint(text, "1;34")
}

pub fn warn(text: impl AsRef<str>) -> String {
    paint(text, "1;33")
}

pub fn format_bps(bps: u16) -> String {
    format!("{bps} ({:.2}%)", f64::from(bps) / 100.0)
}

pub fn format_token_amount(value: U256, decimals: u8) -> String {
    format_units(value, usize::from(decimals)).unwrap_or_else(|_| value.to_string())
}

pub fn format_duration_human(seconds: U256) -> String {
    let raw = seconds.to_string();
    if seconds > U256::from(u64::MAX) {
        return format!("{raw} sec");
    }
    let mut s = seconds.as_u64();
    let days = s / 86_400;
    s %= 86_400;
    let hours = s / 3_600;
    s %= 3_600;
    let minutes = s / 60;
    let secs = s % 60;

    let mut parts: Vec<String> = Vec::new();
    if days > 0 {
        parts.push(format!("{days}d"));
    }
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    if secs > 0 || parts.is_empty() {
        parts.push(format!("{secs}s"));
    }
    format!("{raw} sec ({})", parts.join(" "))
}

pub fn format_status(code: u8) -> &'static str {
    match code {
        0 => "raising",
        1 => "deploying",
        2 => "operating",
        3 => "paused",
        4 => "closed",
        _ => "unknown",
    }
}

pub fn format_ts(ts: U256) -> String {
    if ts > U256::from(u64::MAX) {
        return ts.to_string();
    }
    format!("{}", ts.as_u64())
}
