//! `rustag scan` — scan a stagenet's recorded transactions for exploit
//! signatures (Phase 3, P3.3).

use anyhow::{bail, Result};
use clap::Args;
use console::style;

use rustag_sim::{scan_outcomes, Severity};

use super::{info, load_outcomes, ok, open_store, resolve_record};

#[derive(Args)]
pub struct ScanArgs {
    /// Stagenet to scan (defaults to the only one, if unambiguous).
    #[arg(short, long)]
    pub stagenet: Option<String>,
    /// How many of the most-recent transactions to scan.
    #[arg(long, default_value_t = 1000)]
    pub limit: i64,
    /// Exit non-zero if any finding is at least this severity
    /// (`info`|`low`|`medium`|`high`|`critical`). Useful as a CI gate.
    #[arg(long)]
    pub fail_on: Option<String>,
}

pub async fn run(args: ScanArgs) -> Result<()> {
    let store = open_store().await?;
    let record = resolve_record(&store, args.stagenet.as_deref()).await?;
    let outcomes = load_outcomes(&store, &record.id, args.limit).await?;

    let report = scan_outcomes(&outcomes);
    println!();
    info(format!(
        "scanned {} transaction(s), {} failed",
        report.total_tx, report.failed_tx
    ));

    if report.findings.is_empty() {
        ok("no exploit signatures found");
    } else {
        println!();
        for finding in &report.findings {
            let tag = severity_tag(finding.severity);
            let where_ = finding
                .tx_index
                .map(|i| format!("tx#{i}"))
                .unwrap_or_else(|| "aggregate".to_string());
            println!(
                "  {} {:<26} [{}] {}",
                tag,
                style(&finding.rule).bold(),
                where_,
                finding.detail
            );
        }
        println!();
        info(format!("{} finding(s) total", report.findings.len()));
    }

    if let Some(level) = &args.fail_on {
        let threshold = parse_severity(level)?;
        if report.has_at_least(threshold) {
            bail!("scan found a finding at or above severity `{level}`");
        }
    }
    Ok(())
}

fn parse_severity(s: &str) -> Result<Severity> {
    Ok(match s.to_lowercase().as_str() {
        "info" => Severity::Info,
        "low" => Severity::Low,
        "medium" => Severity::Medium,
        "high" => Severity::High,
        "critical" => Severity::Critical,
        other => bail!("unknown severity `{other}` (use info|low|medium|high|critical)"),
    })
}

fn severity_tag(severity: Severity) -> console::StyledObject<&'static str> {
    match severity {
        Severity::Info => style(" INFO ").dim(),
        Severity::Low => style(" LOW  ").blue(),
        Severity::Medium => style(" MED  ").yellow(),
        Severity::High => style(" HIGH ").red().bold(),
        Severity::Critical => style(" CRIT ").on_red().white().bold(),
    }
}
