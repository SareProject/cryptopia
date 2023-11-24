pub mod error;

use bip39::{Language, Mnemonic};
use ring::hkdf;
use ring::rand::{SecureRandom, SystemRandom};

use crate::error::*;

pub struct Seed {
    raw_seed: [u8; 128],
}

impl Seed {
    pub fn new(raw_seed: [u8; 128]) -> Self {
        Seed { raw_seed }
    }

    pub fn generate() -> Self {
        let rng = SystemRandom::new();
        let mut raw_seed_buffer = [0u8; 128];

        rng.fill(&mut raw_seed_buffer).unwrap();

        Seed {
            raw_seed: raw_seed_buffer,
        }
    }

    pub fn to_mnemonic(&self) -> String {
        let seed_chunks: Vec<&[u8]> = self.raw_seed.chunks_exact(32).collect();

        let mut mnemonic_phrase: String = String::new();

        for chunk in seed_chunks {
            let mnemonic = Mnemonic::from_entropy(&chunk, Language::English).unwrap();
            mnemonic_phrase.push_str(mnemonic.phrase());
            mnemonic_phrase.push(' ');
        }

        mnemonic_phrase.trim_end().to_string()
    }

    pub fn from_mnemonic(seed_phrase: &str) -> Result<Self, SeedError> {
        let phrase_seperated: Vec<&str> = seed_phrase.split_whitespace().collect();

        let mut raw_seed_buffer: Vec<u8> = Vec::new();

        for phrase in phrase_seperated.chunks_exact(24) {
            let mnemonic = Mnemonic::from_phrase(&phrase.join(" "), Language::English);

            if let Ok(mnemonic_parsed) = mnemonic {
                let entropy = mnemonic_parsed.entropy();
                raw_seed_buffer.extend(entropy);
            } else {
                return Err(SeedError::InvalidMnemonicPhrase);
            }
        }

        let raw_seed = <[u8; 128]>::try_from(raw_seed_buffer.as_slice())?;

        Ok(Seed { raw_seed })
    }

    pub fn get_raw_seed(&self) -> [u8; 128] {
        self.raw_seed
    }

    pub fn get_salt_part(&self) -> &[u8] {
        &self.raw_seed[120..]
    }

    pub fn derive_64bytes_child_seed(&self, additional_context: Option<&[&[u8]]>) -> [u8; 64] {
        let salt = hkdf::Salt::new(hkdf::HKDF_SHA512, self.get_salt_part());
        let hkdf_prk = salt.extract(&self.raw_seed);

        let hkdf_okm = hkdf_prk
            .expand(additional_context.unwrap_or(&[&[0]]), hkdf::HKDF_SHA512)
            .unwrap();

        let mut child_seed = [0u8; 64];
        hkdf_okm.fill(&mut child_seed).unwrap();

        child_seed
    }

    pub fn derive_32bytes_child_seed(&self, additional_context: Option<&[&[u8]]>) -> [u8; 32] {
        let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, self.get_salt_part());
        let hkdf_prk = salt.extract(&self.raw_seed);

        let hkdf_okm = hkdf_prk
            .expand(additional_context.unwrap_or(&[&[0]]), hkdf::HKDF_SHA256)
            .unwrap();

        let mut child_seed = [0u8; 32];
        hkdf_okm.fill(&mut child_seed).unwrap();

        child_seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_RAW_SEED: [u8; 128] = [
        107, 77, 197, 228, 108, 243, 219, 99, 78, 6, 163, 49, 167, 215, 3, 169, 28, 225, 74, 229,
        73, 142, 237, 44, 255, 42, 149, 31, 19, 188, 90, 149, 19, 3, 211, 176, 24, 229, 12, 40, 56,
        181, 125, 236, 17, 75, 78, 186, 184, 87, 223, 0, 22, 191, 56, 252, 146, 123, 162, 130, 153,
        100, 84, 12, 121, 96, 192, 255, 68, 161, 170, 4, 151, 183, 68, 143, 22, 140, 68, 157, 95,
        97, 93, 92, 201, 154, 221, 86, 124, 141, 233, 52, 9, 125, 216, 221, 143, 138, 19, 193, 231,
        220, 61, 154, 49, 92, 145, 167, 33, 168, 71, 156, 228, 247, 106, 74, 130, 191, 185, 185,
        118, 141, 70, 127, 85, 252, 241, 113,
    ];

    const TEST_32BYTES_CHILD_SEED: [u8; 32] = [
        146, 250, 163, 138, 246, 233, 76, 112, 125, 255, 167, 171, 59, 186, 57, 138, 182, 2, 176,
        201, 65, 160, 156, 171, 32, 150, 151, 115, 91, 24, 95, 158,
    ];

    const TEST_64BYTES_CHILD_SEED: [u8; 64] = [
        221, 150, 172, 70, 198, 156, 116, 157, 56, 39, 195, 187, 19, 54, 164, 187, 81, 127, 95, 50,
        11, 31, 117, 247, 62, 237, 165, 150, 201, 237, 197, 223, 152, 14, 217, 220, 237, 57, 252,
        202, 141, 47, 164, 50, 20, 179, 148, 89, 44, 227, 146, 61, 84, 40, 59, 176, 27, 102, 241,
        95, 81, 177, 102, 93,
    ];

    const TEST_MNEMONIC_PHRASE: &str = "hero hotel jungle supreme diet random day stamp coyote dirt science fall sock pistol news crack unfold gun skirt clay van taste heart process basic burden ugly crack express beef tissue quick ugly medal squirrel install lyrics usage able subject decline tonight page eagle civil rate expand never just alcohol divert matter boy across gain trigger monitor refuse bachelor deny voyage push industry crew tail recycle casino sponsor dog same gloom phone moon explain vacant soul sense snack shell mutual poet ask ball degree exhaust release claw fitness rifle slight person mind vocal wrist shift clock";

    #[test]
    fn derive_child_seed() {
        let master_seed = Seed::new(TEST_RAW_SEED);

        let child_seed_32bytes = master_seed.derive_32bytes_child_seed(None);
        let child_seed_64bytes = master_seed.derive_64bytes_child_seed(None);

        assert_eq!(child_seed_32bytes, TEST_32BYTES_CHILD_SEED);
        assert_eq!(child_seed_64bytes, TEST_64BYTES_CHILD_SEED);
    }

    #[test]
    fn menmonic_seed_encode() {
        let master_seed = Seed::new(TEST_RAW_SEED);

        let phrase = master_seed.to_mnemonic();

        assert_eq!(phrase, TEST_MNEMONIC_PHRASE);
    }

    #[test]
    fn menmonic_seed_decode() {
        let master_seed = Seed::from_mnemonic(TEST_MNEMONIC_PHRASE).unwrap();

        assert_eq!(master_seed.get_raw_seed(), TEST_RAW_SEED);
    }
}
