//! Peer address types
use std::{
    convert::TryInto,
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
};

use derive_more::{From, Into};
use sigma_ser::{ScorexSerializable, ScorexSerializationError};

/// Peer address
#[derive(PartialEq, Eq, Debug, Copy, Clone, From, Into, Hash)]
pub struct PeerAddr(pub SocketAddr);

impl PeerAddr {
    /// Size in bytes of the ip address associated with this peer address
    pub fn ip_size(&self) -> usize {
        match self.0.ip() {
            IpAddr::V4(ip) => ip.octets().len(),
            IpAddr::V6(ip) => ip.octets().len(),
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
        w.put_u32(self.0.port() as u32)?;

        Ok(())
    }

    fn scorex_parse<R: sigma_ser::vlq_encode::ReadSigmaVlqExt>(
        r: &mut R,
    ) -> Result<Self, sigma_ser::ScorexParsingError> {
        let mut fa = [0u8; 4];
        r.read_exact(&mut fa)?;

        let ip = Ipv4Addr::from(fa);
        let port: u16 = r.get_u32()?.try_into()?;

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
        let obj: PeerAddr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080).into();
        assert_eq![scorex_serialize_roundtrip(&obj), obj]
    }
}
