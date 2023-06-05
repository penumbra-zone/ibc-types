use crate::prelude::*;

/// Packet commitment
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacketCommitment(pub Vec<u8>);

impl PacketCommitment {
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for PacketCommitment {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for PacketCommitment {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

/// Acknowledgement commitment to be stored
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcknowledgementCommitment(pub Vec<u8>);

impl AcknowledgementCommitment {
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for AcknowledgementCommitment {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for AcknowledgementCommitment {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}
