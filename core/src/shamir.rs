use crate::gf256::Gf256;
use rand::Rng;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};
use std::collections::HashSet;

#[derive(Error, Debug)]
pub enum ShamirError {
    #[error("Threshold must be between 2 and 255")]
    InvalidThreshold,
    #[error("Total shares must be >= threshold and <= 255")]
    InvalidTotalShares,
    #[error("Secret cannot be empty")]
    EmptySecret,
    #[error("Duplicate share indices provided")]
    DuplicateShares,
    #[error("Inconsistent share lengths")]
    InconsistentShareLengths,
}

#[derive(Clone, Zeroize, ZeroizeOnDrop, Debug)]
pub struct Share {
    pub index: u8,
    pub data: Vec<u8>,
}

/// Splits a secret into `total_shares`, requiring `threshold` shares to reconstruct.
pub fn split_secret(secret: &[u8], threshold: u8, total_shares: u8) -> Result<Vec<Share>, ShamirError> {
    if threshold < 2 { return Err(ShamirError::InvalidThreshold); }
    if total_shares < threshold { return Err(ShamirError::InvalidTotalShares); }
    if secret.is_empty() { return Err(ShamirError::EmptySecret); }

    let mut rng = rand::thread_rng();
    let mut shares = vec![
        Share { index: 0, data: vec![0; secret.len()] }; 
        total_shares as usize
    ];

    for i in 0..total_shares {
        shares[i as usize].index = i + 1;
    }

    // Process byte by byte to support arbitrary length secrets
    for byte_idx in 0..secret.len() {
        let secret_byte = Gf256(secret[byte_idx]);
        
        // Generate random coefficients. coeffs[0] is the secret byte.
        let mut coeffs: Zeroizing<Vec<Gf256>> = Zeroizing::new(vec![Gf256::ZERO; threshold as usize]);
        coeffs[0] = secret_byte;
        for i in 1..threshold as usize {
            coeffs[i] = Gf256(rng.gen::<u8>());
        }

        // Evaluate the polynomial for each share
        for i in 0..total_shares as usize {
            let x = Gf256(shares[i].index);
            let y = Gf256::evaluate_polynomial(&coeffs, x);
            shares[i].data[byte_idx] = y.0;
        }
    }

    Ok(shares)
}

/// Recovers the original secret from a slice of shares using Lagrange Interpolation.
pub fn recover_secret(shares: &[Share]) -> Result<Zeroizing<Vec<u8>>, ShamirError> {
    if shares.is_empty() { return Err(ShamirError::EmptySecret); }
    let secret_len = shares[0].data.len();
    if secret_len == 0 { return Err(ShamirError::EmptySecret); }

    let mut indices = HashSet::new();
    for s in shares {
        if s.data.len() != secret_len { return Err(ShamirError::InconsistentShareLengths); }
        if !indices.insert(s.index) { return Err(ShamirError::DuplicateShares); }
    }

    let mut secret = Zeroizing::new(vec![0u8; secret_len]);

    // Recover byte by byte
    for byte_idx in 0..secret_len {
        let mut recovered = Gf256::ZERO;
        
        for (i, share_i) in shares.iter().enumerate() {
            let x_i = Gf256(share_i.index);
            let y_i = Gf256(share_i.data[byte_idx]);
            
            let mut num = Gf256::ONE;
            let mut den = Gf256::ONE;
            
            // Calculate the Lagrange basis polynomial l_i(0)
            for (j, share_j) in shares.iter().enumerate() {
                if i == j { continue; }
                let x_j = Gf256(share_j.index);
                num = num * x_j;
                den = den * (x_j - x_i);
            }
            
            let term = y_i * num * den.inverse();
            recovered = recovered + term;
        }
        
        secret[byte_idx] = recovered.0;
    }

    Ok(secret)
}
