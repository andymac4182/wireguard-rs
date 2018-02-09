use failure::{Error, SyncFailure};
use snow::{NoiseBuilder, Session};
use snow::params::NoiseParams;
use types::{InterfaceInfo, PeerInfo};


lazy_static! {
    static ref NOISE_PARAMS: NoiseParams = "Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s".parse().unwrap();
}

/// Wrapper around the `snow` library to easily setup the handshakes for WireGuard.
pub struct Noise {}
impl Noise {
    fn new_foundation<'a>(local_privkey: &'a [u8]) -> NoiseBuilder<'a> {
        NoiseBuilder::new(NOISE_PARAMS.clone())
            .local_private_key(local_privkey)
            .prologue("WireGuard v1 zx2c4 Jason@zx2c4.com".as_bytes())
    }

    pub fn build_initiator(local_privkey: &[u8], remote_pubkey: &[u8], psk: &Option<[u8; 32]>) -> Result<Session, Error> {
        Ok(Noise::new_foundation(local_privkey)
            .remote_public_key(remote_pubkey)
            .psk(2, psk.as_ref().unwrap_or_else(|| &[0u8; 32]))
            .build_initiator()
            .map_err(SyncFailure::new)?)
    }

    pub fn build_responder(local_privkey: &[u8]) -> Result<Session, Error> {
        Ok(Noise::new_foundation(local_privkey)
            .build_responder()
            .map_err(SyncFailure::new)?)

    }
}