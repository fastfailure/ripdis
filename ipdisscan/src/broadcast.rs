use crate::conf::ScannerConfig;
use color_eyre::eyre::Report;
use crossbeam::channel::TryRecvError;
use crossbeam::channel::{bounded, Receiver, Sender};
use ipdisserver::signature::Signature;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use tracing::{debug, info, instrument, trace};

const SCANNER_ADDR: Ipv4Addr = Ipv4Addr::UNSPECIFIED; // "0.0.0.0"

#[instrument]
pub fn run(
    socket: &UdpSocket,
    new_beacon_notification_channel_recv_end: Receiver<()>,
    conf: &ScannerConfig,
) -> Result<(), Report> {
    let mut empty_scans = 0; // incremented up to max_empty_scans+1
    let max_empty_scans = 10; // delay before slowing down scanning
    let slowdown_factor = 10.0; // slowing down x10 broadcasts if no new answer is received
    {
        info!(?socket, base_frequency=1.0/conf.scan_period, ?conf.signatures, "Scanning for beacons.");
        loop {
            send_single(
                socket,
                conf.broadcast_addr,
                conf.target_port,
                &conf.signatures,
            )?;
            let scan_period = if empty_scans > max_empty_scans {
                conf.scan_period * slowdown_factor
            } else {
                conf.scan_period
            };
            wait_duty_cycle(scan_period);
            match new_beacon_notification_channel_recv_end.try_recv() {
                Err(TryRecvError::Empty) => match empty_scans {
                    x if x < max_empty_scans => {
                        empty_scans += 1;
                    }
                    x if x == max_empty_scans => {
                        debug!("No new beacon answer received for {} scan cycles, slowing down scan period.", max_empty_scans);
                        empty_scans += 1;
                    }
                    _ => {}
                },
                Ok(_) => {
                    trace!("Answer from new beacon received, scan period shortened.");
                    empty_scans = 0;
                }
                Err(TryRecvError::Disconnected) => {
                    return Err(Report::msg("notification channel disconnected"))
                }
            }
        }
    }
}

pub fn socket_setup(scanner_port: u16) -> Result<UdpSocket, Report> {
    let socket = UdpSocket::bind(format!("{}:{}", SCANNER_ADDR, scanner_port))
        .expect("Failed to setup broadcasting socket");
    socket.set_broadcast(true)?;
    Ok(socket)
}

pub fn init_notification_channel() -> (Sender<()>, Receiver<()>) {
    bounded(1)
}

#[instrument]
fn send_single(
    socket: &UdpSocket,
    broadcast_addr: Ipv4Addr,
    target_port: u16,
    signatures: &[Signature],
) -> Result<(), Report> {
    let beacon_broadcast_addr = SocketAddr::from((broadcast_addr, target_port));
    for signature in signatures {
        socket
            .send_to(&signature.0, beacon_broadcast_addr)
            .expect("Failed broadcasting signature");
        trace!(
            dest = %beacon_broadcast_addr,
            payload = ?signature.0,
            "Broadcasted."
        );
    }
    Ok(())
}

fn wait_duty_cycle(scan_period: f64) {
    thread::sleep(Duration::from_secs_f64(scan_period));
}

#[cfg(test)]
mod test {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    #[tracing_test::traced_test]
    fn test_send() {
        let signature: Signature = Signature::from("test-signature");
        let signatures = vec![signature.clone()];
        let listener_socket = UdpSocket::bind(format!("{}:{}", "0.0.0.0", 0)).unwrap();
        let mut buf = [0; 14];
        let listener_port = listener_socket.local_addr().unwrap().port();

        let sender_handle = thread::spawn(move || {
            thread::sleep(Duration::from_secs_f64(0.1));
            let socket = socket_setup(1902).unwrap();
            send_single(&socket, Ipv4Addr::BROADCAST, listener_port, &signatures).unwrap();
        });

        let (lenght, _source) = listener_socket.recv_from(&mut buf).unwrap();
        assert_eq!(lenght, signature.0.len());
        assert_eq!(buf.to_vec(), signature.0);
        sender_handle.join().unwrap();
    }
}
