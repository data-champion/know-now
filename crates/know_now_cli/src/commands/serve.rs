use std::io::IsTerminal;
use std::net::IpAddr;

use know_now_server::ServerConfig;

use crate::context::CommandContext;

#[derive(Debug, clap::Args)]
pub struct ServeArgs {
    /// Host address to bind to
    #[arg(long, default_value = "127.0.0.1")]
    pub host: IpAddr,

    /// Port to bind to
    #[arg(long, default_value = "3827")]
    pub port: u16,

    /// Enable write endpoints (generation trigger)
    #[arg(long)]
    pub allow_generate: bool,

    /// Required with --allow-generate when --host is not localhost
    #[arg(long)]
    pub allow_generate_on_network: bool,

    /// Do not auto-open the dashboard in a browser at startup
    #[arg(long)]
    pub no_browser: bool,
}

pub fn run(ctx: &CommandContext, args: &ServeArgs) -> anyhow::Result<()> {
    let config = ServerConfig {
        host: args.host,
        port: args.port,
        allow_generate: args.allow_generate,
        project_root: ctx.project_root.clone(),
        persist_launch_info: true,
    };

    if !config.is_localhost() {
        eprintln!(
            "WARNING: server is binding to {} — this server is not intended as an \
             authenticated multi-user deployment.",
            args.host
        );
    }

    if args.allow_generate && !config.is_localhost() && !args.allow_generate_on_network {
        anyhow::bail!(
            "--allow-generate on a non-localhost address requires --allow-generate-on-network"
        );
    }

    let auto_open = should_auto_open(args.no_browser);

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let handle = know_now_server::start_server(config).await?;

        println!("know-now server running at {}", handle.url);
        println!("Open in browser: {}", handle.launch_url);

        if auto_open {
            if let Err(e) = opener::open(&handle.launch_url) {
                eprintln!(
                    "warning: failed to auto-open browser ({e}); navigate to the URL above manually"
                );
            }
        }

        tokio::signal::ctrl_c().await.ok();
        println!("\nShutting down...");
        handle.shutdown();

        Ok(())
    })
}

fn should_auto_open(no_browser_flag: bool) -> bool {
    if no_browser_flag {
        return false;
    }
    if std::env::var_os("CI").is_some() {
        return false;
    }
    std::io::stdout().is_terminal()
}
