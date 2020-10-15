use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use chacha20poly1305::aead::{Aead, NewAead, Payload};

use crate::commands::hash::UserPrivateKey;

///
/// Шифрование пакета
///
/// - проверка правильности дешифрации
/// - проверка открытых данных (aead)
///
///
#[derive(Clone)]
pub struct Cipher<'a> {
	private_key: &'a UserPrivateKey,
}

impl<'a> Cipher<'a> {
	pub fn new(private_key: &'a UserPrivateKey) -> Self {
		Self {
			private_key,
		}
	}
	
	pub fn encrypt(&mut self, msg: &[u8], ad: &[u8], nonce: [u8; 8]) -> Vec<u8> {
		let mut nonce_buffer = [0; 12];
		nonce_buffer[0..8].copy_from_slice(&nonce);
		let key = Key::from_slice(self.private_key);
		let nonce = Nonce::from_slice(&nonce_buffer);
		let cipher = ChaCha20Poly1305::new(key);
		cipher.encrypt(nonce, Payload {
			msg,
			aad: ad,
		}).unwrap()
	}
	
	pub fn decrypt(&mut self, msg: &[u8], ad: &[u8], nonce: [u8; 8]) -> Result<Vec<u8>, ()> {
		let mut nonce_buffer = [0; 12];
		nonce_buffer[0..8].copy_from_slice(&nonce);
		let key = Key::from_slice(self.private_key);
		let nonce = Nonce::from_slice(&nonce_buffer);
		let cipher = ChaCha20Poly1305::new(key);
		cipher.decrypt(nonce, Payload {
			msg,
			aad: ad,
		}).map_err(|_| ())
	}
}

#[cfg(test)]
mod tests {
	use easybench::bench;
	use crate::udp::protocol::codec::cipher::Cipher;
	
	const PRIVATE_KEY: &[u8; 32] = &[
		0x29, 0xfa, 0x35, 0x60, 0x88, 0x45, 0xc6, 0xf9,
		0xd8, 0xfe, 0x65, 0xe3, 0x22, 0x0e, 0x5b, 0x05,
		0x03, 0x4a, 0xa0, 0x9f, 0x9e, 0x27, 0xad, 0x0f,
		0x6c, 0x90, 0xa5, 0x73, 0xa8, 0x10, 0xe4, 0x94,
	];
	const ORIGINAL: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
	const NONCE: [u8; 8] = [0; 8];
	const AD: [u8; 4] = [1, 2, 3, 4];
	const OTHER_AD: [u8; 2] = [0, 1];
	
	///
	/// 806 ns - libsodium
	/// 825 ns - chacha20poly1305 = "0.6.0"
	///
	#[test]
	fn bench_cipher() {
		let aad = vec![0, 1, 2, 3];
		println!("{}", bench(|| {
			let mut cipher = Cipher::new(PRIVATE_KEY);
			let encrypted = cipher.encrypt(&ORIGINAL, &AD, NONCE);
			cipher.decrypt(&encrypted, &*aad, NONCE).unwrap();
		}))
	}
	
	#[test]
	fn should_cipher() {
		let mut cipher = Cipher::new(PRIVATE_KEY);
		let encrypted = cipher.encrypt(&ORIGINAL, &AD, NONCE);
		let decrypted = cipher.decrypt(&encrypted, &AD, NONCE).unwrap();
		assert_eq!(decrypted, ORIGINAL);
		assert_ne!(decrypted, encrypted);
	}
	
	#[test]
	fn should_fail_when_different_ad() {
		let mut cipher = Cipher::new(PRIVATE_KEY);
		let encrypted = cipher.encrypt(&ORIGINAL, &AD, NONCE);
		assert!(matches!(cipher.decrypt(&encrypted, &OTHER_AD, NONCE), Result::Err(())));
	}
	
	#[test]
	fn should_fail_when_broken_packet() {
		let mut cipher = Cipher::new(PRIVATE_KEY);
		let mut encrypted = cipher.encrypt(&ORIGINAL, &AD, NONCE);
		encrypted[0] = 0;
		assert!(matches!(cipher.decrypt(&encrypted, &OTHER_AD, NONCE), Result::Err(())));
	}
}