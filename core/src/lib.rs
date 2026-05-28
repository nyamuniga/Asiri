pub mod gf256;
pub mod shamir;

pub use shamir::{split_secret, recover_secret, Share, ShamirError};

#[cfg(test)]
mod tests {
    use super::*;
    use gf256::Gf256;

    #[test]
    fn test_gf256_math() {
        // Simple test for GF(2^8)
        let a = Gf256(0x53);
        let b = Gf256(0xCA);
        // Add
        assert_eq!(a + b, Gf256(0x99));
        // Mul
        assert_eq!(a * b, Gf256(0x01)); // 0x53 and 0xCA are inverses in AES GF(2^8)
        // Inv
        assert_eq!(a.inverse(), b);
        assert_eq!(b.inverse(), a);
    }

    #[test]
    fn test_shamir_split_recover() {
        let secret = b"super secret message that is longer than 255 bytes potentially... wait this is shorter, but it proves it handles bytes! \x00\x01\x02";
        
        // Split into 5 shares, require 3
        let shares = split_secret(secret, 3, 5).unwrap();
        assert_eq!(shares.len(), 5);

        // Try to recover with exactly 3 shares (0, 2, 4)
        let subset = vec![shares[0].clone(), shares[2].clone(), shares[4].clone()];
        let recovered = recover_secret(&subset).unwrap();
        assert_eq!(recovered.as_slice(), secret);

        // Try to recover with 2 shares (should fail or yield gibberish)
        let subset_bad = vec![shares[0].clone(), shares[1].clone()];
        let recovered_bad = recover_secret(&subset_bad).unwrap();
        assert_ne!(recovered_bad.as_slice(), secret);
    }
}
