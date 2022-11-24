mod setup;

use clap::Parser;
use color_eyre::eyre::Report;
use ipdisscan::conf::{
    ScannerConfig, BROADCAST_ADDR_DEFAULT, EXTRA_SIGNATURE_DEFAULT, SCANNER_PORT_DEFAULT,
    SCAN_PERIOD_DEFAULT,
};
use ipdisscan::{
    beacons,
    broadcast::{self, socket_setup},
    listen, ui,
};
use ipdisserver::{Signature, SERVER_PORT_DEFAULT, SIGNATURE_DEFAULT};
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::thread;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// UDP port used to receive ipdisserver answers.
    /// Default: 1902.
    #[arg(short, long, default_value_t = SCANNER_PORT_DEFAULT)]
    port: u16,

    /// Broadcasting address. Default is the limited broadcast address: 255.255.255.255. You can
    /// also use any subnet-directed broadcast address, e.g. 192.168.1.255 (for the network
    /// 192.168.1.0/24).
    #[arg(short, long, default_value_t = BROADCAST_ADDR_DEFAULT)]
    broadcast_addr: Ipv4Addr,

    /// ipdisserver listening UDP port. Default: 1901.
    #[arg(short, long, default_value_t = SERVER_PORT_DEFAULT)]
    target_port: u16,

    /// String used to recognize ipdisserver instances. UTF-8 characters are allowed.
    /// Signature length must be 128 bytes at most.
    /// Default (NB: multiple signatures):
    /// `ipdisbeacon` and `pang-supremacy-maritime-revoke-afterglow`
    /// (the second one is for backward compatibility).
    #[arg(short, long)]
    signature: Option<String>,

    #[arg(long, default_value_t = SCAN_PERIOD_DEFAULT)]
    scan_period: f64,

    /// File where logs will be emitted.
    /// By default it's $TMPDIR/ipdisscan.log or /tmp/ipdisscan.log on Linux.
    /// Set to /dev/null to suppress logs.
    // Cannot emit logs to stderr, it would destroy the UI!
    #[arg(short, long)]
    log_file: Option<PathBuf>,
}

fn main() -> Result<(), Report> {
    setup::eyre_setup()?;
    let cli = Cli::parse();
    let signatures: Vec<Signature> = match cli.signature.as_deref() {
        Some(s) => vec![Signature::from(s)],
        None => vec![
            Signature::from(SIGNATURE_DEFAULT),
            Signature::from(EXTRA_SIGNATURE_DEFAULT),
        ],
    };
    let log_file = cli.log_file.unwrap_or_else(ScannerConfig::default_log_file);
    let conf = ScannerConfig {
        port: cli.port,
        scan_period: cli.scan_period,
        broadcast_addr: cli.broadcast_addr,
        target_port: cli.target_port,
        log_file,
        signatures,
    };
    setup::log_setup(&conf.log_file)?;

    let socket = socket_setup(conf.port)?;
    let socket_c = socket.try_clone()?;
    let (input_channel_send_end, input_channel_receive_end) = beacons::init_input_channel();
    let (output_channel_send_end, output_channel_receive_end) = beacons::init_output_channel();
    let (new_beacon_notification_channel_send_end, new_beacon_notification_channel_receive_end) =
        broadcast::init_notification_channel();
    thread::spawn(move || listen::run(&socket_c, input_channel_send_end));
    thread::spawn(move || {
        broadcast::run(&socket, new_beacon_notification_channel_receive_end, &conf)
    });
    thread::spawn(move || {
        beacons::run(
            input_channel_receive_end,
            output_channel_send_end,
            new_beacon_notification_channel_send_end,
        )
    });
    ui::run(output_channel_receive_end)?;
    Ok(())
}
