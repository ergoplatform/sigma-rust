//! Peer address types
use std::{
    // convert::TryInto,
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
};

use derive_more::{From, Into};
use sigma_ser::{ScorexSerializable, ScorexSerializationError};

/// Peer address errors
pub enum PeerAddrError {
    /// Ipv6 is not currently supported
    Ipv6Unsupported,
}

/// Peer address
#[derive(PartialEq, Eq, Debug, Copy, Clone, From, Into)]
pub struct PeerAddr(pub SocketAddr);

impl PeerAddr {
    /// Size in bytes of the ip address associated with this peer address
    pub fn ip_size(&self) -> Result<usize, PeerAddrError> {
        let ip = self.0.ip();

        match ip {
            IpAddr::V4(ip) => Ok(ip.octets().len()),
            IpAddr::V6(_) => Err(PeerAddrError::Ipv6Unsupported),
        }
    }
}

impl From<PeerAddrError> for io::Error {
    fn from(e: PeerAddrError) -> Self {
        match e {
            PeerAddrError::Ipv6Unsupported => {
                io::Error::new(io::ErrorKind::Unsupported, "ipv6 not supported")
            }
        }
    }
}

impl ScorexSerializable for PeerAddr {
    fn scorex_serialize<W: sigma_ser::vlq_encode::WriteSigmaVlqExt>(
        &self,
        w: &mut W,
    ) -> sigma_ser::ScorexSerializeResult {
        let ip = match self.0.ip() {
            IpAddr::V4(ip) => ip,
            _ => return Err(ScorexSerializationError::NotSupported("ipv6 not supported")),
        };

        w.write_all(&ip.octets())?;
        w.put_u16(self.0.port())?;

        Ok(())
    }

    fn scorex_parse<R: sigma_ser::vlq_encode::ReadSigmaVlqExt>(
        r: &mut R,
    ) -> Result<Self, sigma_ser::ScorexParsingError> {
        let mut fa = [0u8; 4];
        r.read_exact(&mut fa)?;

        let ip = Ipv4Addr::from(fa);
        let port = r.get_u16()?;

        Ok(SocketAddr::V4(SocketAddrV4::new(ip, port)).into())
    }
}

#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;
    use sigma_ser::scorex_serialize_roundtrip;

    #[test]
    fn ser_roundtrip() {
        let obj: PeerAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080).into();
        assert_eq![scorex_serialize_roundtrip(&obj), obj]
    }
}
