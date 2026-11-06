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
    Tools {
        /// Base URL of gateway
        #[arg(short, long, default_value = "http://localhost:9090")]
        url: String,
        /// Auth token
        #[arg(long)]
        token: Option<String>,
    },
    /// Execute a tool by name
    ToolsExecute {
        /// Base URL of gateway
        #[arg(short, long, default_value = "http://localhost:9090")]
        url: String,
        /// Tool name
        #[arg(short, long)]
        tool: String,
        /// JSON args
        #[arg(short, long)]
        args: Option<String>,
        /// Auth token
        #[arg(long)]
        token: Option<String>,
    },
    /// Show or update gateway config
    Config {
        /// Config file path
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// Set a key=value override
        #[arg(long)]
        set: Vec<String>,
    },
    /// Terminal chat client: send stdin lines to gateway
    Chat {
        /// Base URL of gateway
        #[arg(short, long, default_value = "http://localhost:9090")]
        url: String,
        /// Channel name
        #[arg(short, long)]
        channel: String,
        /// Chat/recipient ID
        #[arg(short, long)]
        to: String,
        /// Auth token
        #[arg(long)]
        token: Option<String>,
    },
}
