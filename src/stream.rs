//! `stream.rs`: The STREAM online authenticated encryption construction.
//! See <https://eprint.iacr.org/2015/189.pdf> for definition.

use crate::{Aead, Aes128PmacSivAead, Aes128SivAead, Aes256PmacSivAead, Aes256SivAead, Error};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Size of a nonce required by STREAM in bytes
pub const NONCE_SIZE: usize = 8;

/// Byte flag indicating this is the last block in the STREAM (otherwise 0)
const LAST_BLOCK_FLAG: u8 = 1;

/// A STREAM encryptor with a 32-bit counter, generalized for any AEAD algorithm
///
/// This corresponds to the ℰ stream encryptor object as defined in the paper
/// Online Authenticated-Encryption and its Nonce-Reuse Misuse-Resistance
pub struct Encryptor<A: Aead> {
    alg: A,
    nonce: NonceEncoder32,
}

/// AES-CMAC-SIV STREAM encryptor with 256-bit key size (128-bit security)
/// and a 64-bit (8-byte) nonce.
pub type Aes128SivEncryptor = Encryptor<Aes128SivAead>;

/// AES-CMAC-SIV STREAM encryptor with 512-bit key size (256-bit security)
/// and a 64-bit (8-byte) nonce.
pub type Aes256SivEncryptor = Encryptor<Aes256SivAead>;

/// AES-PMAC-SIV STREAM encryptor with 256-bit key size (128-bit security)
/// and a 64-bit (8-byte) nonce.
pub type Aes128PmacSivEncryptor = Encryptor<Aes128PmacSivAead>;

/// AES-PMAC-SIV STREAM encryptor with 512-bit key size (256-bit security)
/// and a 64-bit (8-byte) nonce.
pub type Aes256PmacSivEncryptor = Encryptor<Aes256PmacSivAead>;

impl<A: Aead> Encryptor<A> {
    /// Create a new STREAM encryptor, initialized with a given key and nonce.
    ///
    /// Panics if the key or nonce is the wrong size.
    pub fn new(key: &[u8], nonce: &[u8]) -> Self {
        Self {
            alg: A::new(key),
            nonce: NonceEncoder32::new(nonce),
        }
    }

    /// Encrypt the next message in the stream in-place
    pub fn encrypt_next_in_place(&mut self, ad: &[u8], buffer: &mut [u8]) {
        self.alg.encrypt_in_place(self.nonce.as_slice(), ad, buffer);
        self.nonce.increment();
    }

    /// Encrypt the final message in-place, consuming the stream encryptor
    pub fn encrypt_last_in_place(mut self, ad: &[u8], buffer: &mut [u8]) {
        self.alg.encrypt_in_place(&self.nonce.finish(), ad, buffer);
    }

    /// Encrypt the next message in the stream, allocating and returning a
    /// `Vec<u8>` for the ciphertext
    #[cfg(feature = "alloc")]
    pub fn encrypt_next(&mut self, ad: &[u8], plaintext: &[u8]) -> Vec<u8> {
        let ciphertext = self.alg.encrypt(self.nonce.as_slice(), ad, plaintext);
        self.nonce.increment();
        ciphertext
    }

    /// Encrypt the final message in the stream, allocating and returning a
    /// `Vec<u8>` for the ciphertext
    #[cfg(feature = "alloc")]
    pub fn encrypt_last(mut self, ad: &[u8], plaintext: &[u8]) -> Vec<u8> {
        self.alg.encrypt(&self.nonce.finish(), ad, plaintext)
    }
}

/// A STREAM decryptor with a 32-bit counter, generalized for any AEAD algorithm
///
/// This corresponds to the 𝒟 stream decryptor object as defined in the paper
/// Online Authenticated-Encryption and its Nonce-Reuse Misuse-Resistance
pub struct Decryptor<A: Aead> {
    alg: A,
    nonce: NonceEncoder32,
}

/// AES-CMAC-SIV STREAM decryptor with 256-bit key size (128-bit security)
/// and a 64-bit (8-byte) nonce.
pub type Aes128SivDecryptor = Decryptor<Aes128SivAead>;

/// AES-CMAC-SIV STREAM decryptor with 512-bit key size (256-bit security)
/// and a 64-bit (8-byte) nonce.
pub type Aes256SivDecryptor = Decryptor<Aes256SivAead>;

/// AES-PMAC-SIV STREAM decryptor with 256-bit key size (128-bit security)
/// and a 64-bit (8-byte) nonce.
pub type Aes128PmacSivDecryptor = Decryptor<Aes128PmacSivAead>;

/// AES-PMAC-SIV STREAM decryptor with 512-bit key size (256-bit security)
/// and a 64-bit (8-byte) nonce.
pub type Aes256PmacSivDecryptor = Decryptor<Aes256PmacSivAead>;

impl<A: Aead> Decryptor<A> {
    /// Create a new STREAM decryptor, initialized with a given key and nonce.
    ///
    /// Panics if the key or nonce is the wrong size.
    pub fn new(key: &[u8], nonce: &[u8]) -> Self {
        Self {
            alg: A::new(key),
            nonce: NonceEncoder32::new(nonce),
        }
    }

    /// Decrypt the next message in the stream in-place
    pub fn decrypt_next_in_place<'a>(
        &mut self,
        ad: &[u8],
        buffer: &'a mut [u8],
    ) -> Result<&'a [u8], Error> {
        let result = self
            .alg
            .decrypt_in_place(self.nonce.as_slice(), ad, buffer)?;
        self.nonce.increment();
        Ok(result)
    }

    /// Decrypt the final message in-place, consuming the stream decryptor
    pub fn decrypt_last_in_place<'a>(
        mut self,
        ad: &[u8],
        buffer: &'a mut [u8],
    ) -> Result<&'a [u8], Error> {
        self.alg.decrypt_in_place(&self.nonce.finish(), ad, buffer)
    }

    /// Decrypt the next message in the stream, allocating and returning a
    /// `Vec<u8>` for the plaintext
    #[cfg(feature = "alloc")]
    pub fn decrypt_next(&mut self, ad: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        let plaintext = self.alg.decrypt(self.nonce.as_slice(), ad, ciphertext)?;
        self.nonce.increment();
        Ok(plaintext)
    }

    /// Decrypt the next message in the stream, allocating and returning a
    /// `Vec<u8>` for the plaintext
    #[cfg(feature = "alloc")]
    pub fn decrypt_last(mut self, ad: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        self.alg.decrypt(&self.nonce.finish(), ad, ciphertext)
    }
}

/// STREAM nonce including space for 32-bit counter and 1-byte last block flag
type StreamNonce = [u8; NONCE_SIZE + 4 + 1];

/// Computes STREAM nonces based on the current position in the STREAM.
///
/// Accepts a 64-bit nonce and uses a 32-bit counter internally.
///
/// Panics if the nonce size is incorrect, 32-bit counter overflows
struct NonceEncoder32 {
    value: StreamNonce,
    counter: u32,
}

impl NonceEncoder32 {
    /// Create a new nonce encoder object
    fn new(prefix: &[u8]) -> Self {
        if prefix.len() != NONCE_SIZE {
            panic!(
                "incorrect nonce size (expected {}, got {})",
                NONCE_SIZE,
                prefix.len()
            );
        }

        let mut result = Self {
            value: Default::default(),
            counter: 0,
        };

        result.value[..NONCE_SIZE].copy_from_slice(prefix);
        result
    }

    /// Increment the nonce value in-place
    pub fn increment(&mut self) {
        self.counter = self
            .counter
            .checked_add(1)
            .expect("STREAM nonce counter overflowed");

        self.value[NONCE_SIZE..(NONCE_SIZE + 4)].copy_from_slice(&self.counter.to_be_bytes());
    }

    /// Borrow the current value as a slice
    pub fn as_slice(&self) -> &[u8] {
        &self.value
    }

    /// Compute the final nonce value, consuming self and returning the final
    /// nonce value.
    pub fn finish(mut self) -> StreamNonce {
        *self.value.iter_mut().last().unwrap() = LAST_BLOCK_FLAG;
        self.value
    }
}
