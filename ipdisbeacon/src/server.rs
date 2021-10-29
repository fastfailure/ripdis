use color_eyre::Report;
// use bytes::{BytesMut, BufMut};
use crate::bytes::{Answer, Signature};
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use tracing::{debug, info, trace};

pub const SERVER_PORT: u16 = 1901;
pub const SIGNATURE_DEFAULT: Signature =
    Signature("pang-supremacy-maritime-revoke-afterglow".as_bytes()); // must be shorter than RECV_BUFFER_LENGHT

const RECV_BUFFER_LENGHT: usize = 64;
const LISTENING_ADDR: &str = "0.0.0.0";
const REFRACTORY_PERIOD_MS: u64 = 3000; // milliseconds, needed to reduce useless communications and to allow every beacon to be polled in a crowded network

pub fn run() -> Result<(), Report> {
    {
        let socket = UdpSocket::bind(format!("{}:{}", LISTENING_ADDR, SERVER_PORT))?;
        info!(?socket, "Listening for scanner requests.");
        loop {
            serve_single(&socket, None)?;
            thread::sleep(Duration::from_millis(REFRACTORY_PERIOD_MS));
        }
    } // the socket is closed here
}

fn get_answer() -> Result<Answer, Report> {
    let answer = Answer(Vec::new());
    Ok(answer)
}

fn serve_single(socket: &UdpSocket, signature: Option<Signature>) -> Result<(), Report> {
    let expected_signature = match signature {
        None => SIGNATURE_DEFAULT,
        Some(s) => s,
    };
    let (addr, received_signature) = receive(socket)?;
    if received_signature.0 != expected_signature.0 {
        debug!(%received_signature, %addr, "Bad signature received, not answering.");
        return Ok(());
    };
    let answer = get_answer()?;
    respond(socket, &addr, &answer)?;
    info!(%answer, %addr, "Answered.");
    Ok(())
}

fn receive(socket: &UdpSocket) -> Result<(SocketAddr, Answer), Report> {
    // Receives a single datagram message on the socket. If `buf` is too small to hold
    // the message, it will be cut off.
    let mut buf = [0; RECV_BUFFER_LENGHT];
    trace!(?socket, "Listening.");
    let (lenght, source) = socket.recv_from(&mut buf)?;
    let received: Answer = (&buf[..lenght]).into();
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
    use std::thread;
    use std::time::Duration;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn test_serve_localhost() {
        let sending_socket = UdpSocket::bind(format!("{}:{}", "0.0.0.0", 1902)).unwrap();
        let receiving_socket = sending_socket
            .try_clone()
            .expect("couldn't clone the socket");
        let beacon_socket = UdpSocket::bind(format!("{}:{}", LISTENING_ADDR, SERVER_PORT)).unwrap();
        let server_handle = thread::spawn(move || {
            serve_single(&beacon_socket, None).unwrap();
        });
        let scanner_handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            let beacon_addr = SocketAddr::from(([127, 0, 0, 1], SERVER_PORT));
            sending_socket
                .send_to(SIGNATURE_DEFAULT.0, beacon_addr)
                .unwrap();
            println!("[{}] <- {}", beacon_addr, SIGNATURE_DEFAULT);
        });
        let response = receive(&receiving_socket).unwrap();
        println!("[{}] -> {}", response.0, response.1);
        server_handle.join().unwrap();
        scanner_handle.join().unwrap();
    }
}
