use fips204::ml_dsa_65; // Matches Dilithium3 security level (approx)
use fips204::traits::{KeyGen, Signer, Verifier, SerDes};
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;
use log::{info, warn};

pub struct Ledger {
    log_file: String,
    // Holding keys in memory for this session
    sk: ml_dsa_65::PrivateKey, 
    pk: ml_dsa_65::PublicKey,
}

impl Ledger {
    pub fn new(filename: &str) -> Self {
        // Generate Real Post-Quantum Keys
        let (pk, sk) = ml_dsa_65::KG::try_keygen().expect("Failed to generate FIPS 204 keys");
        
        info!("FIPS 204/ML-DSA Keys Generated.");
        // pk.clone() to avoid move, or just omit logging the bytes if too expensive. 
        // For debugging, we clone.
        info!("Public Key (First 16 bytes): {}", hex::encode(&pk.clone().into_bytes()[0..16]));

        Self {
            log_file: filename.to_string(),
            sk,
            pk,
        }
    }

    pub fn record_transaction(&self, price: f64, theta: f64, job_id: &str) {
        let timestamp = Utc::now().to_rfc3339();
        let payload = format!("{}|{}|{}|{}", timestamp, price, theta, job_id);
        let payload_bytes = payload.as_bytes();
        let ctx = b"sentinel-ctx"; // Context string required by FIPS 204 standard
        
        // 1. Sign (Real Math)
        let signature = self.sk.try_sign(payload_bytes, ctx).expect("Signing failed");
        
        // 2. Verify (Immediate Correctness Check)
        let valid = self.pk.verify(payload_bytes, &signature, ctx);
        if !valid {
             warn!("CRITICAL: FIPS 204 Signature Verification Failed internally!");
        }

        // Signature is an array [u8; N], not a struct with into_bytes() in some versions, 
        // or it implements generic trait. fips204 0.4.6 Signature is likely a byte array or has to_vec.
        // The error said `into_bytes` not found for array `[u8; 3309]`. So it returned an array directly.
        let sig_hex = hex::encode(signature); 

        // 3. Persist
        let entry = format!("{}|{}\n", payload, sig_hex);

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&self.log_file) {
            if let Err(e) = file.write_all(entry.as_bytes()) {
                eprintln!("Failed to write to ledger: {}", e);
            }
        }
    }
}
