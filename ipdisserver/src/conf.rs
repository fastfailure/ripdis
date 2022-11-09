use crate::Signature;
use color_eyre::eyre::Report;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::net::Ipv4Addr;
use std::path::Path;
use tracing::info;

pub const SERVER_PORT_DEFAULT: u16 = 1901;
pub const SIGNATURE_DEFAULT: &str = "ipdisbeacon"; // must be shorter than RECV_BUFFER_LENGHT
pub const LISTENING_ADDR_DEFAULT: Ipv4Addr = Ipv4Addr::UNSPECIFIED; // "0.0.0.0"

/// Server configurations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig<'a> {
    pub port: u16,
    pub listening_addr: Ipv4Addr,
    pub signatures: Vec<Signature>,
    pub inventory_files: Vec<&'a Path>,
}

impl ServerConfig<'_> {
    /// Read a sequence of Signature from a file, one per line.
    /// Empty lines are ignored.
    pub fn parse_signatures_file(path: &Path) -> Result<Vec<Signature>, Report> {
        info!(?path, "Reading signatures from file.");
        let mut signatures = Vec::new();
        for line in read_file_lines(path)? {
            match line?.as_str() {
                "" => continue,
                s => signatures.push(Signature::from(s)),
            };
        }
        Ok(signatures)
    }

    pub fn dummy() -> Self {
        let port = SERVER_PORT_DEFAULT;
        let listening_addr = LISTENING_ADDR_DEFAULT;
        let signatures = vec![SIGNATURE_DEFAULT.into()];
        let inventory_files = Vec::new();
        Self {
            port,
            listening_addr,
            signatures,
            inventory_files,
        }
    }
}

/// Returns an Iterator to the Reader of the lines of the file.
/// The output is wrapped in a Result to allow matching on errors
fn read_file_lines<P>(filename: P) -> io::Result<Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[tracing_test::traced_test]
    fn test_parse_signature_file() {
        let datadir = std::env::temp_dir()
            .as_path()
            .join("rust-ipdisserver-test-conf-datadir/");
        // TODO: windows
        if let Err(error) = std::fs::create_dir(&datadir) {
            match error.kind() {
                std::io::ErrorKind::AlreadyExists => (),
                _ => panic!(),
            }
        };
        let empty_file_path = datadir.join("empty-file");
        let sign_file_path = datadir.join("sign-file");
        std::fs::write(&empty_file_path, "").unwrap();
        std::fs::write(&sign_file_path, "TestSignature\n\nsign line 2\n\n\n").unwrap();

        assert_eq!(
            ServerConfig::parse_signatures_file(&empty_file_path).unwrap(),
            Vec::new()
        );
        assert_eq!(
            ServerConfig::parse_signatures_file(&sign_file_path).unwrap(),
            vec![
                Signature::from("TestSignature"),
                Signature::from("sign line 2")
            ]
        );
    }
}
