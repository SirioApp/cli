use crate::config::RuntimeConfig;
use crate::types::ProjectView;
use ethers::prelude::{TransactionReceipt, U256};
use ethers::utils::format_units;

pub fn print_network(cfg: &RuntimeConfig) {
    println!("network: {:?}", cfg.network);
    println!("label: {}", cfg.network_label);
    println!("chain_id: {}", cfg.chain_id);
    println!("rpc: {}", cfg.rpc_url);
    println!("factory: {:?}", cfg.factory);
    println!("allowlist: {:?}", cfg.allowlist);
    println!(
        "default_collateral: {}",
        cfg.default_collateral
            .map(|a| format!("{:?}", a))
            .unwrap_or_else(|| "<none>".to_string())
    );
    println!("deployment_file: {}", cfg.deployment_path.display());
}

pub fn print_receipt(receipt: &TransactionReceipt) {
    println!("tx_hash: {:?}", receipt.transaction_hash);
    println!("block: {}", receipt.block_number.unwrap_or_default());
    println!("status: {}", receipt.status.unwrap_or_default());
}

pub fn print_project(id: u64, project: &ProjectView, approved: bool) {
    println!("---");
    println!("project_id: {}", id);
    println!("name: {}", project.name);
    println!("description: {}", project.description);
    if !project.categories.is_empty() {
        println!("categories: {}", project.categories);
    }
    println!("agent_id: {}", project.agent_id);
    println!("agent: {:?}", project.agent);
    println!("treasury: {:?}", project.treasury);
    println!("sale: {:?}", project.sale);
    println!("agent_executor: {:?}", project.agent_executor);
    println!("collateral: {:?}", project.collateral);
    println!(
        "operational_status: {} ({})",
        project.operational_status,
        format_status(project.operational_status)
    );
    if !project.status_note.is_empty() {
        println!("status_note: {}", project.status_note);
    }
    println!("approved: {}", approved);
    println!("created_at: {}", format_ts(project.created_at));
    println!("updated_at: {}", format_ts(project.updated_at));
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
    ts.as_u64().to_string()
}

fn supports_color() -> bool {
    std::env::var_os("NO_COLOR").is_none()
        && std::env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
}

fn paint(text: impl AsRef<str>, code: &str) -> String {
    let text = text.as_ref();
    if supports_color() {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}
