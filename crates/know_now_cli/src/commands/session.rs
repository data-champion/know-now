use know_now_server::launch_info;

use crate::context::CommandContext;

#[derive(Debug, clap::Args)]
pub struct SessionUrlArgs {
    /// Rewrite the URL's scheme/host/port (e.g., http://localhost:5173 for
    /// cross-origin dev with a vite proxy). The `/__open?launch_token=...`
    /// path/query is preserved.
    #[arg(long)]
    pub origin: Option<String>,

    /// Output as JSON for tooling consumption
    #[arg(long)]
    pub json: bool,
}

pub fn run(ctx: &CommandContext, args: &SessionUrlArgs) -> anyhow::Result<()> {
    let info = launch_info::read_launch_info(&ctx.project_root).map_err(|e| {
        anyhow::anyhow!(
            "failed to read .knownow/launch.json: {e}\n\nIs `know-now serve` running in this project?"
        )
    })?;

    let url = match args.origin.as_deref() {
        Some(origin) => retarget_url(&info.token, origin)?,
        None => info.url.clone(),
    };

    if args.json {
        let value = serde_json::json!({
            "url": url,
            "token": info.token,
            "host": info.host,
            "port": info.port,
            "scheme": info.scheme,
        });
        println!("{}", serde_json::to_string_pretty(&value)?);
    } else {
        println!("{url}");
    }

    Ok(())
}

fn retarget_url(token: &str, origin: &str) -> anyhow::Result<String> {
    let trimmed = origin.trim_end_matches('/');
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        anyhow::bail!("--origin must start with http:// or https://; got: {origin}");
    }
    Ok(format!("{trimmed}/__open?launch_token={token}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retarget_basic() {
        let url = retarget_url("abc-123", "http://localhost:5173").unwrap();
        assert_eq!(url, "http://localhost:5173/__open?launch_token=abc-123");
    }

    #[test]
    fn retarget_strips_trailing_slash() {
        let url = retarget_url("tok", "http://localhost:5173/").unwrap();
        assert_eq!(url, "http://localhost:5173/__open?launch_token=tok");
    }

    #[test]
    fn retarget_rejects_missing_scheme() {
        let err = retarget_url("tok", "localhost:5173").unwrap_err();
        assert!(err.to_string().contains("must start with http://"));
    }
}
