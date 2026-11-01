mod gateway;

use gateway::{
    cli::{Cli, Commands},
    server::serve,
    GatewayConfig, GatewayClient,
};
use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Serve { listen, .. } => {
            let mut cfg = GatewayConfig::default();
            if let Some(addr) = listen {
                cfg.listen = addr;
            }
            serve(cfg).await?;
            Ok(())
        }
        Commands::Send { channel, to, text, .. } => {
            println!("send scaffold: {} -> {}: {}", channel, to, text);
            Ok(())
        }
        Commands::Tools => {
            println!("tools scaffold");
            Ok(())
        }
    }
}
