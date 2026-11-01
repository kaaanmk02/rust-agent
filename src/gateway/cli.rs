use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Gateway HTTP server
    Serve {
        /// Listen address, e.g. 0.0.0.0:9090
        #[arg(short, long)]
        listen: Option<String>,
        /// Config file path
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Send a message via gateway client
    Send {
        /// Base URL of gateway
        #[arg(short, long)]
        url: String,
        /// Channel name
        #[arg(short, long)]
        channel: String,
        /// Chat/recipient ID
        #[arg(short, long)]
        to: String,
        /// Text message
        #[arg(short, long)]
        text: String,
        /// Auth token
        #[arg(long)]
        token: Option<String>,
    },
    /// List available tools
    Tools,
}
