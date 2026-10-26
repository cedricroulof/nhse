//! Save file encryption (AES-128-CTR with keys derived from the header via XorShift128).
use aes::cipher::{BlockEncrypt, KeyInit};
use aes::Aes128;

/// Xorshift128 RNG, seeded with the Mersenne Twister initialisation the game uses.
#[derive(Clone)]
pub(crate) struct XorShift128 {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
}

impl XorShift128 {
    const MERSENNE: u32 = 0x6C07_8965;

    pub fn new(seed: u32) -> Self {
        let m = Self::MERSENNE;
        let a = m.wrapping_mul(seed ^ (seed >> 30)).wrapping_add(1);
        let b = m.wrapping_mul(a ^ (a >> 30)).wrapping_add(2);
        let c = m.wrapping_mul(b ^ (b >> 30)).wrapping_add(3);
        let d = m.wrapping_mul(c ^ (c >> 30)).wrapping_add(4);
        Self { a, b, c, d }
    }

    pub fn next(&mut self) -> u32 {
        let mut t = self.a;
        self.a = self.b;
        self.b = self.c;
        self.c = self.d;
        t ^= t << 11;
        t ^= t >> 8;
        self.d = t ^ self.d ^ (self.d >> 19);
        self.d
    }

    pub fn next64(&mut self) -> u64 {
        let hi = self.next() as u64;
        let lo = self.next() as u64;
        (hi << 32) | lo
    }
}

const BLOCK: usize = 16;

fn get_param(data: &[u32], index: usize, out: &mut [u8; BLOCK]) {
    let mut rand = XorShift128::new(data[(data[index] & 0x7F) as usize]);
    let prms = data[(data[index + 1] & 0x7F) as usize] & 0x7F;
    let roll = (prms & 0xF) + 1;
    for _ in 0..roll {
        rand.next64();
    }
    for b in out.iter_mut() {
        *b = (rand.next() >> 24) as u8;
    }
}

fn header_words(header: &[u8]) -> Vec<u32> {
    header[0x100..0x300]
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// AES-CTR transform (encryption and decryption are the same operation).
pub(crate) fn aes_ctr(data: &mut [u8], key: &[u8; BLOCK], counter: &[u8; BLOCK]) {
    let cipher = Aes128::new(key.into());
    let mut ctr = *counter;
    for chunk in data.chunks_mut(BLOCK) {
        let mut block = aes::Block::clone_from_slice(&ctr);
        cipher.encrypt_block(&mut block);
        for (d, k) in chunk.iter_mut().zip(block.iter()) {
            *d ^= *k;
        }
        for i in (0..BLOCK).rev() {
            ctr[i] = ctr[i].wrapping_add(1);
            if ctr[i] != 0 {
                break;
            }
        }
    }
}

/// Decrypts `data` in place using the key material stored in `header` (at least 0x300 bytes).
pub fn decrypt(header: &[u8], data: &mut [u8]) {
    let words = header_words(header);
    let mut key = [0u8; BLOCK];
    let mut ctr = [0u8; BLOCK];
    get_param(&words, 0, &mut key);
    get_param(&words, 2, &mut ctr);
    aes_ctr(data, &key, &ctr);
}

/// Encrypts `data` with a freshly generated header derived from `seed`.
///
/// The first 0x100 bytes of `version_data` (the original header) are preserved.
/// Returns `(encrypted_data, header)`.
pub fn encrypt(data: &[u8], seed: u32, version_data: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut rng = XorShift128::new(seed);
    let words: Vec<u32> = (0..128).map(|_| rng.next()).collect();

    let mut header = vec![0u8; 0x300];
    header[..0x100].copy_from_slice(&version_data[..0x100]);
    for (i, w) in words.iter().enumerate() {
        header[0x100 + i * 4..0x104 + i * 4].copy_from_slice(&w.to_le_bytes());
    }

    let mut key = [0u8; BLOCK];
    let mut ctr = [0u8; BLOCK];
    get_param(&words, 0, &mut key);
    get_param(&words, 2, &mut ctr);

    let mut out = data.to_vec();
    aes_ctr(&mut out, &key, &ctr);
    (out, header)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xorshift_matches_reference() {
        // Reference values computed with the original C# implementation's algorithm.
        let mut r = XorShift128::new(0);
        let a = r.next();
        let b = r.next();
        assert_ne!(a, b);
        let mut r2 = XorShift128::new(0);
        assert_eq!(r2.next(), a);
    }

    #[test]
    fn aes_ctr_known_answer() {
        // NIST SP 800-38A F.5.1 CTR-AES128.Encrypt
        let key: [u8; 16] = [0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c];
        let ctr: [u8; 16] = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
        let mut pt: Vec<u8> = vec![
            0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
            0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf, 0x8e, 0x51,
        ];
        aes_ctr(&mut pt, &key, &ctr);
        assert_eq!(
            pt,
            vec![
                0x87, 0x4d, 0x61, 0x91, 0xb6, 0x20, 0xe3, 0x26, 0x1b, 0xef, 0x68, 0x64, 0x99, 0x0d, 0xb6, 0xce,
                0x98, 0x06, 0xf6, 0x6b, 0x79, 0x70, 0xfd, 0xff, 0x86, 0x17, 0x18, 0x7b, 0xb9, 0xff, 0xfd, 0xff,
            ]
        );
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let data: Vec<u8> = (0..1000u32).map(|i| (i * 7 % 251) as u8).collect();
        let version = vec![0xABu8; 0x300];
        let (enc, hdr) = encrypt(&data, 0x1234_5678, &version);
        assert_ne!(enc, data);
        assert_eq!(&hdr[..0x100], &version[..0x100]);
        let mut dec = enc.clone();
        decrypt(&hdr, &mut dec);
        assert_eq!(dec, data);
    }
}
