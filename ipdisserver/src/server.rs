use crate::bytes::{Answer, BeaconInfos, Signature};
use crate::conf::ServerConfig;
use color_eyre::Report;
use gethostname::gethostname;
use serde_json;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use tracing::{debug, info, trace};

const RECV_BUFFER_LENGHT: usize = 64;
const REFRACTORY_PERIOD: f64 = 3.0; // needed to reduce useless communications and to allow every beacon to be polled in a crowded network

pub fn run(conf: &ServerConfig) -> Result<(), Report> {
    {
        let socket = UdpSocket::bind(format!("{}:{}", conf.listening_addr, conf.port))?;
        info!(?socket, "Listening for scanner requests.");
        loop {
            serve_single(&socket, &conf.signatures)?;
            thread::sleep(Duration::from_secs_f64(REFRACTORY_PERIOD));
        }
    } // the socket is closed here
}

fn get_hostname() -> String {
    gethostname().to_string_lossy().into()
}

fn get_answer() -> Result<Answer, Report> {
    let hostname = BeaconInfos::String(get_hostname());
    let hostname_key = "hostname".to_string();
    let mut hostname_answer = serde_json::map::Map::new();
    hostname_answer.insert(hostname_key, hostname);
    let basic_answer = BeaconInfos::Object(hostname_answer);
    let answer = Answer::from(serde_json::to_string(&basic_answer)?);
    Ok(answer)
}

fn serve_single(socket: &UdpSocket, expected_signatures: &[Signature]) -> Result<(), Report> {
    let (addr, received) = receive(socket)?;
    if !is_signature_vaid(&received, expected_signatures) {
        debug!(%received, %addr, "Bad signature received, not answering.");
        return Ok(());
    };
    let answer = get_answer()?;
    respond(socket, &addr, &answer)?;
    info!(%answer, %addr, "Answered.");
    Ok(())
}

fn is_signature_vaid(received: &Signature, expected: &[Signature]) -> bool {
    trace!(%received, ?expected, "Validating received signature.");
    for signature in expected.iter() {
        if signature == received {
            trace!(%received, "Received signature matches.");
            return true;
        }
    }
    false
}

fn receive(socket: &UdpSocket) -> Result<(SocketAddr, Signature), Report> {
    // Receives a single datagram message on the socket. If `buf` is too small to hold
    // the message, it will be cut off.
    let mut buf = [0; RECV_BUFFER_LENGHT];
    trace!(?socket, "Listening.");
    let (lenght, source) = socket.recv_from(&mut buf)?;
    let received: Signature = (&buf[..lenght]).into();
    trace!(%lenght, %source, "Datagram received.");
    Ok((source, received))
}

fn respond(socket: &UdpSocket, addr: &SocketAddr, msg: &Answer) -> Result<(), Report> {
    socket.send_to(&msg.0, addr)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::net::Ipv4Addr;
    use std::thread;
    use std::time::Duration;

    #[test]
    #[tracing_test::traced_test]
    fn test_serve_localhost() {
        let conf = ServerConfig::default();
        let sending_socket = UdpSocket::bind(format!("{}:{}", Ipv4Addr::UNSPECIFIED, 0)).unwrap();
        let receiving_socket = sending_socket
            .try_clone()
            .expect("couldn't clone the socket");
        let beacon_socket = UdpSocket::bind(format!("{}:{}", conf.listening_addr, 0)).unwrap();
        let server_port = beacon_socket.local_addr().unwrap().port();
        let conf_clone = conf.clone();
        let server_handle = thread::spawn(move || {
            serve_single(&beacon_socket, &conf_clone.signatures).unwrap();
        });
        let scanner_handle = thread::spawn(move || {
            thread::sleep(Duration::from_secs_f64(0.1));
            let beacon_addr = SocketAddr::from(([127, 0, 0, 1], server_port));
            sending_socket
                .send_to(conf.signatures.first().unwrap().0.as_ref(), beacon_addr)
                .unwrap();
            println!("[{}] <- {:?}", beacon_addr, &conf.signatures);
        });
        let response = receive(&receiving_socket).unwrap();
        println!("[{}] -> {}", response.0, response.1);
        server_handle.join().unwrap();
        scanner_handle.join().unwrap();
    }
}
