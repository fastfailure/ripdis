use color_eyre::eyre::Report;
use crossbeam::channel::{unbounded, Receiver, Sender, TrySendError};
use ipdisserver::answers::Answer;
use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;
use tracing::{instrument, trace};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaconAnswer {
    pub addr: IpAddr,
    pub payload: Answer,
}

impl fmt::Display for BeaconAnswer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.addr, self.payload)
    }
}

type BeaconAnswers = HashMap<IpAddr, BeaconAnswer>;

#[instrument]
pub fn run(
    channel_receiving_end: Receiver<BeaconAnswer>,
    output_channel_send_end: Sender<Vec<BeaconAnswer>>,
    new_beacon_notification_channel_send_end: Sender<()>,
) -> Result<(), Report> {
    let mut servers = BeaconAnswers::new();
    trace!("Starting server answers update loop.");
    loop {
        servers = beacons_update(
            servers,
            channel_receiving_end.clone(),
            new_beacon_notification_channel_send_end.clone(),
        )?;
        output_channel_send_end.try_send(servers.values().map(|x| x.to_owned()).collect())?;
    }
}

pub fn init_input_channel() -> (Sender<BeaconAnswer>, Receiver<BeaconAnswer>) {
    unbounded()
}

pub fn init_output_channel() -> (Sender<Vec<BeaconAnswer>>, Receiver<Vec<BeaconAnswer>>) {
    unbounded()
}

#[instrument]
fn beacons_update(
    mut beacons: BeaconAnswers,
    channel_receiving_end: Receiver<BeaconAnswer>,
    new_beacon_notification_channel_send_end: Sender<()>,
) -> Result<BeaconAnswers, Report> {
    loop {
        let beacon = match channel_receiving_end.try_recv() {
            Ok(b) => b,
            _ => return Ok(beacons),
        };
        trace!(?beacon, "Updating beacons.");
        if beacons.insert(beacon.addr, beacon).is_none() {
            trace!("New beacon added.");
            if let Err(TrySendError::Disconnected(_)) =
                new_beacon_notification_channel_send_end.try_send(())
            {
                return Err(Report::msg("notification channel disconnected"));
            }
            // capacity is 1, it's OK if it's full
        } else {
            trace!("Updating already known beacon.");
        }
    }
}

#[cfg(test)]
mod test {
    use crate::broadcast::init_notification_channel;

    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    #[tracing_test::traced_test]
    fn test_beacons_update() {
        let (sender, receiver) = init_input_channel();
        let (notifier, _notif_recv) = init_notification_channel();
        let answer1 = BeaconAnswer {
            addr: IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)),
            payload: Answer::default(),
        };
        let answer1_new = BeaconAnswer {
            addr: IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)),
            payload: Answer::default(),
        };
        let answer2 = BeaconAnswer {
            addr: IpAddr::V4(Ipv4Addr::new(192, 168, 0, 2)),
            payload: Answer::default(),
        };
        let answer2_new = BeaconAnswer {
            addr: IpAddr::V4(Ipv4Addr::new(192, 168, 0, 2)),
            payload: Answer::default(),
        };
        sender.send(answer2.clone()).unwrap();
        sender.send(answer1.clone()).unwrap();
        sender.send(answer1_new.clone()).unwrap();
        sender.send(answer2_new.clone()).unwrap();
        let mut beacons = BeaconAnswers::new();
        beacons = beacons_update(beacons, receiver, notifier).unwrap();
        assert_eq!(
            beacons.get(&answer1.addr).unwrap().payload,
            answer1_new.payload
        );
        assert_eq!(
            beacons.get(&answer2.addr).unwrap().payload,
            answer2_new.payload
        );
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_put_in_queue() {
        let (sender, receiver) = init_input_channel();
        let an_answer = BeaconAnswer {
            addr: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            payload: Answer::default(),
        };
        sender.send(an_answer.clone()).unwrap();
        assert_eq!(receiver.try_recv().unwrap(), an_answer);
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_init_in_queue() {
        let (_sender, receiver) = init_input_channel();
        assert!(receiver.is_empty());
    }
}
