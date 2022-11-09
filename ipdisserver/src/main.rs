mod setup;

use clap::Parser;
use color_eyre::eyre::Report;
use ipdisserver::conf::{
    ServerConfig, LISTENING_ADDR_DEFAULT, SERVER_PORT_DEFAULT, SIGNATURE_DEFAULT,
};
use ipdisserver::{server, Signature};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

// use color_eyre::{eyre::WrapErr};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Listening port. Default: 1901.
    #[arg(short, long, default_value_t = SERVER_PORT_DEFAULT)]
    port: u16,

    /// Listening address. Default: 0.0.0.0
    #[arg(short, long, default_value_t = LISTENING_ADDR_DEFAULT)]
    addr: Ipv4Addr,

    /// Path of a file with accepted signatures, one per line. UTF-8 characters are allowed. Each
    /// signature length must be 128 bytes at most. If not specified a single signature is
    /// accepted: `ipdisbeacon`.
    #[arg(short, long)]
    signatures_file: Option<PathBuf>,

    /// Send logs to systemd-journald instead of stderr.
    #[arg(short = 'j', long)]
    journald: bool,

    /// r#"Specify a list of files to execute, the output will be added to the answer. The output must be in the format `key0=value0\nkey1=value1\n...`. Repeat the option for each file."#
    #[arg(short = 'f', long, action = clap::ArgAction::Append, value_hint = clap::ValueHint::FilePath)]
    inventory: Vec<PathBuf>,
}

fn main() -> Result<(), Report> {
    let cli = Cli::parse();
    let do_log_to_journald = cli.journald;
    setup::setup(do_log_to_journald)?;
    let signatures = match cli.signatures_file {
        Some(p) => ServerConfig::parse_signatures_file(&p)?,
        None => vec![Signature::from(SIGNATURE_DEFAULT)],
    };
    info!("Accepted signatures: {:?}", signatures);
    let inventory_files: Vec<&Path> = Vec::new();
    let conf = ServerConfig {
        port: cli.port,
        listening_addr: cli.addr,
        signatures,
        inventory_files,
    };
    debug!("Starting IP discovery server.");
    server::run(&conf)?;
    Ok(())
}
