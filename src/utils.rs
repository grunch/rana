use bech32::{ToBase32, Variant};
use qrcode::{render::unicode, QrCode};
use std::{error::Error, time::Instant};

use nostr_sdk::prelude::Secp256k1;
use secp256k1::rand::thread_rng;

/// Benchmark the cores capabilities for key generation
pub fn benchmark_cores(cores: usize, pow_difficulty: u8) {
    let mut hashes_per_second_per_core = 0;

    println!("Benchmarking a single core for 5 seconds...");
    let now = Instant::now();
    let secp = Secp256k1::new();
    let mut rng = thread_rng();
    loop {
        let (_secret_key, public_key) = secp.generate_keypair(&mut rng);
        let (xonly_public_key, _) = public_key.x_only_public_key();
        get_leading_zero_bits(&xonly_public_key.serialize());
        hashes_per_second_per_core += 1;
        if now.elapsed().as_secs() > 5 {
            break;
        }
    }
    hashes_per_second_per_core /= 10;
    println!("A single core can mine roughly {hashes_per_second_per_core} h/s!");

    let estimated_hashes = 2_u128.pow(pow_difficulty as u32);
    println!("Searching for prefix of {pow_difficulty} specific bits");
    let estimate = estimated_hashes as f32 / hashes_per_second_per_core as f32 / cores as f32;
    println!("This is estimated to take about {estimate} seconds");
}

/// Print private and public keys to the output
pub fn print_keys(
    secret_key: String,
    xonly_public_key: String,
    vanity_npub: String,
    leading_zeroes: u8,
    mnemonic: Option<String>,
) -> Result<(), Box<dyn Error>> {
    if leading_zeroes != 0 {
        println!("Leading zero bits:         {leading_zeroes}");
    } else if !vanity_npub.is_empty() {
        println!("Vanity npub found:         {vanity_npub}")
    }

    println!("Found matching public key: {xonly_public_key}");

    println!("Nostr private key: {secret_key:>72}");

    println!(
        "Nostr public key (npub): {:>65}",
        bech32::encode(
            "npub",
            hex::decode(xonly_public_key)?.to_base32(),
            Variant::Bech32
        )?
    );
    println!(
        "Nostr private key (nsec): {:>64}",
        bech32::encode(
            "nsec",
            hex::decode(secret_key)?.to_base32(),
            Variant::Bech32
        )?
    );
    if let Some(mnemonic_value) = mnemonic {
        println!("Mnemonic:                  {}", mnemonic_value);
    }

    Ok(())
}

#[inline]
pub fn get_leading_zero_bits(bytes: &[u8]) -> u8 {
    let mut res = 0_u8;
    for b in bytes {
        if *b == 0 {
            res += 8;
        } else {
            res += b.leading_zeros() as u8;
            return res;
        }
    }
    res
}

pub fn print_qr(secret_key_string: String) -> Result<(), Box<dyn Error>> {
    let private_hex = secret_key_string;
    let nsec = bech32::encode(
        "nsec",
        hex::decode(private_hex)?.to_base32(),
        Variant::Bech32,
    )?;
    let code = QrCode::new(nsec).unwrap();
    let qr = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    println!("{qr}");
    Ok(())
}
