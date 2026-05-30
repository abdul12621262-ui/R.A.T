//! E2E session encryption using X25519 + ChaCha20-Poly1305.

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::rngs::OsRng;
use thiserror::Error;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

const NONCE_LEN: usize = 12;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("decryption failed")]
    Decrypt,
    #[error("invalid public key length")]
    InvalidPublicKey,
    #[error("handshake already complete")]
    HandshakeDone,
}

pub struct SessionCrypto {
    cipher: ChaCha20Poly1305,
    send_nonce: u64,
    recv_nonce: u64,
    pending_secret: Option<EphemeralSecret>,
}

impl SessionCrypto {
    /// Initiator: returns public key to send via signaling.
    pub fn initiator() -> (Self, Vec<u8>) {
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        let placeholder = ChaCha20Poly1305::new(&[0u8; 32].into());
        (
            Self {
                cipher: placeholder,
                send_nonce: 0,
                recv_nonce: 0,
                pending_secret: Some(secret),
            },
            public.as_bytes().to_vec(),
        )
    }

    pub fn complete_handshake(&mut self, peer_public: &[u8]) -> Result<(), CryptoError> {
        let secret = self
            .pending_secret
            .take()
            .ok_or(CryptoError::HandshakeDone)?;
        let peer = parse_public(peer_public)?;
        let shared: SharedSecret = secret.diffie_hellman(&peer);
        self.cipher = ChaCha20Poly1305::new(&derive_key(shared.as_bytes()).into());
        Ok(())
    }

    pub fn responder(peer_public: &[u8]) -> Result<(Self, Vec<u8>), CryptoError> {
        let peer = parse_public(peer_public)?;
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        let shared = secret.diffie_hellman(&peer);
        Ok((
            Self {
                cipher: ChaCha20Poly1305::new(&derive_key(shared.as_bytes()).into()),
                send_nonce: 0,
                recv_nonce: 0,
                pending_secret: None,
            },
            public.as_bytes().to_vec(),
        ))
    }

    pub fn encrypt(&mut self, plaintext: &[u8]) -> Vec<u8> {
        let nonce = self.next_send_nonce();
        let mut out = nonce.to_vec();
        let ciphertext = self
            .cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext)
            .expect("encrypt");
        out.extend(ciphertext);
        out
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if data.len() < NONCE_LEN {
            return Err(CryptoError::Decrypt);
        }
        let (nonce_bytes, ct) = data.split_at(NONCE_LEN);
        let plaintext = self
            .cipher
            .decrypt(Nonce::from_slice(nonce_bytes), ct)
            .map_err(|_| CryptoError::Decrypt)?;
        self.recv_nonce += 1;
        Ok(plaintext)
    }

    fn next_send_nonce(&mut self) -> [u8; NONCE_LEN] {
        let mut n = [0u8; NONCE_LEN];
        n[4..].copy_from_slice(&self.send_nonce.to_le_bytes());
        self.send_nonce += 1;
        n
    }
}

fn parse_public(peer_public: &[u8]) -> Result<PublicKey, CryptoError> {
    if peer_public.len() != 32 {
        return Err(CryptoError::InvalidPublicKey);
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(peer_public);
    Ok(PublicKey::from(arr))
}

fn derive_key(shared: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, b) in shared.iter().enumerate() {
        out[i % 32] ^= b.wrapping_mul((i as u8).wrapping_add(1));
    }
    for b in shared.iter().rev() {
        out.rotate_left(1);
        out[0] ^= *b;
    }
    out
}
