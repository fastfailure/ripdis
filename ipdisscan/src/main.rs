use clap::{App, Arg};
use color_eyre::eyre::Report;
use ipdisscan::beacons;
use ipdisscan::broadcast;
use ipdisscan::broadcast::socket_setup;
use ipdisscan::conf::ScannerConfig;
use ipdisscan::listen;
use ipdisscan::setup::setup;
use ipdisscan::ui;
use ipdisserver::signature::Signature;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::thread;
use tracing::trace;

fn main() -> Result<(), Report> {
    const PORT_OPT: &str = "port";
    const TARGET_PORT_OPT: &str = "target_port";
    const ADDR_OPT: &str = "addr";
    const SIGNATURE_OPT: &str = "signatures";
    let matches = App::new("ipdisscan")
        .version("0.1.1")
        .about("Search for active instances of ipdisserver and get system informations.")
        .arg(
            Arg::with_name(PORT_OPT)
                .short("p")
                .long("scanner-source-port")
                .value_name("PORT")
                .help("UDP port used to receive ipdisserver answers. Default: 1902.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(TARGET_PORT_OPT)
                .short("b")
                .long("broadcast-target-port")
                .value_name("TARGET-PORT")
                .help("ipdisserver listening UDP port. Default: 1901.")
                .takes_value(true),
        )
        .arg(
    Arg::with_name(ADDR_OPT)
                .short("a")
                .long("broadcast-addr")
                .value_name("ADDR")
                .help("Broadcasting address. Default is the limited broadcast address: 255.255.255.255. You can also use any subnet-directed broadcast address, e.g. 192.168.1.255 (for the network 192.168.1.0/24).")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(SIGNATURE_OPT)
                .short("s")
                .long("signature")
                .value_name("SIGN")
                .multiple(true)
                .number_of_values(1)
                .help("Strings used to recognize ipdisserver instances. UTF-8 characters are allowed. Each signature length must be 128 bytes at most. This option can be used more than once. Default: `ipdisbeacon` and `pang-supremacy-maritime-revoke-afterglow` (the second one is for backward compatibility).")
                .takes_value(true),
        )
        .get_matches();

    setup()?;
    trace!(?matches);

    let mut conf = ScannerConfig::default();
    if matches.is_present(PORT_OPT) {
        conf.port = matches.value_of(PORT_OPT).unwrap().parse()?;
    }
    if matches.is_present(TARGET_PORT_OPT) {
        conf.target_port = matches.value_of(TARGET_PORT_OPT).unwrap().parse()?;
    }
    if matches.is_present(ADDR_OPT) {
        let str_broadcast_addr = matches.value_of(ADDR_OPT).unwrap().parse::<String>()?;
        conf.broadcast_addr = Ipv4Addr::from_str(&str_broadcast_addr)?;
    }
    if matches.is_present(SIGNATURE_OPT) {
        conf.signatures = matches
            .values_of(SIGNATURE_OPT)
            .unwrap()
            .into_iter()
            .map(Signature::from)
            .collect();
        // replace default signatures
    }

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
