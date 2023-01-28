use bech32::{ToBase32, Variant};
use bitcoin_hashes::hex::ToHex;
use clap::Parser;
use rana::cli::*;
use secp256k1::rand::thread_rng;
use secp256k1::{Secp256k1, SecretKey, XOnlyPublicKey};
use std::cmp::max;
use std::error::Error;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

const DIFFICULTY_DEFAULT: u8 = 10;

fn main() -> Result<(), Box<dyn Error>> {
    // Parse CLI arguments

    let parsed_args = CLIArgs::parse();

    let mut difficulty = parsed_args.difficulty;
    let vanity_prefix = parsed_args.vanity_prefix;
    let mut vanity_npub_prefixes = <Vec<String>>::new();
    let mut vanity_npub_suffixes = <Vec<String>>::new();
    let num_cores = parsed_args.num_cores;

    for vanity_npub_pre in parsed_args.vanity_npub_prefixes_raw_input.split(',') {
        if !vanity_npub_pre.is_empty() {
            vanity_npub_prefixes.push(vanity_npub_pre.to_string())
        }
    }
    for vanity_npub_post in parsed_args.vanity_npub_suffixes_raw_input.split(',') {
        if !vanity_npub_post.is_empty() {
            vanity_npub_suffixes.push(vanity_npub_post.to_string())
        }
    }

    check_args(difficulty, vanity_prefix.as_str(), &vanity_npub_prefixes, &vanity_npub_suffixes, num_cores);

    //-- Calculate pow difficulty and initialize

    // initially the same as difficulty
    let mut pow_difficulty = difficulty;

    if !vanity_prefix.is_empty() {
        // set pow difficulty as the length of the prefix translated to bits
        pow_difficulty = (vanity_prefix.len() * 4) as u8;
        println!(
            "Started mining process for vanity hex prefix: '{}' (estimated pow: {})",
            vanity_prefix, pow_difficulty
        );

    } else if !vanity_npub_prefixes.is_empty() {
        // set pow difficulty as the length of the first prefix translated to bits
        pow_difficulty = (vanity_npub_prefixes[0].len() * 4) as u8;
        println!(
            "Started mining process for vanity bech32 prefix[es]: 'npub1{:?}' (estimated pow: {})",
            vanity_npub_prefixes, pow_difficulty
        );

    } else {
        // Defaults to using difficulty

        // if difficulty not indicated, then assume default
        if difficulty == 0 {
            difficulty = DIFFICULTY_DEFAULT; // default
            pow_difficulty = difficulty;
        }

        println!(
            "Started mining process with a difficulty of: {difficulty} (pow: {})",
            pow_difficulty
        );
    }

    // benchmark cores
    if !vanity_npub_prefixes.is_empty() || !vanity_npub_suffixes.is_empty() {
        println!("Benchmarking of cores disabled for vanity npub key upon proper calculation.");
    } else {
        benchmark_cores(num_cores, pow_difficulty);
    }

    // Loop: generate public keys until desired public key is reached
    let now = Instant::now();

    println!("Mining using {num_cores} cores...");

    // thread safe variables
    let best_diff = Arc::new(AtomicU8::new(pow_difficulty));
    let vanity_ts = Arc::new(vanity_prefix);
    let vanity_npubs_pre_ts = Arc::new(vanity_npub_prefixes);
    let vanity_npubs_post_ts = Arc::new(vanity_npub_suffixes);
    let iterations = Arc::new(AtomicU64::new(0));

    // start a thread for each core for calculations
    for _ in 0..num_cores {
        let best_diff = best_diff.clone();
        let vanity_ts = vanity_ts.clone();
        let vanity_npubs_pre_ts = vanity_npubs_pre_ts.clone();
        let vanity_npubs_post_ts = vanity_npubs_post_ts.clone();
        let iterations = iterations.clone();
        thread::spawn(move || {
            let mut rng = thread_rng();
            let secp = Secp256k1::new();
            loop {
                iterations.fetch_add(1, Ordering::Relaxed);

                let (secret_key, public_key) = secp.generate_keypair(&mut rng);
                let (xonly_public_key, _) = public_key.x_only_public_key();

                let mut leading_zeroes = 0;
                let mut vanity_npub = "".to_string();

                // check pubkey validity depending on arg settings
                let mut is_valid_pubkey: bool = false;
                let hexa_key = xonly_public_key.to_hex();

                if vanity_ts.as_str() != "" {
                    // hex vanity search
                    is_valid_pubkey = hexa_key.starts_with(vanity_ts.as_str());

                } else if !vanity_npubs_pre_ts.is_empty() || !vanity_npubs_post_ts.is_empty() {
                    // bech32 vanity search
                    let bech_key: String = bech32::encode(
                        "npub",
                        hex::decode(hexa_key).unwrap().to_base32(),
                        Variant::Bech32,
                    )
                    .unwrap();

                    if !vanity_npubs_pre_ts.is_empty() && !vanity_npubs_post_ts.is_empty() {
                        for cur_vanity_npub_pre in vanity_npubs_pre_ts.iter() {
                            for cur_vanity_npub_post in vanity_npubs_post_ts.iter() {
                                is_valid_pubkey = bech_key.starts_with(
                                    (String::from("npub1") + cur_vanity_npub_pre.as_str()).as_str(),
                                ) && bech_key.ends_with(cur_vanity_npub_post.as_str());

                                if is_valid_pubkey {
                                    vanity_npub = cur_vanity_npub_pre.clone() + "..." + cur_vanity_npub_post.clone().as_str();
                                    break;
                                }
                            }
                            if is_valid_pubkey {break;}
                        }
                    }
                    else if !vanity_npubs_pre_ts.is_empty() {
                        for cur_vanity_npub in vanity_npubs_pre_ts.iter() {
                            is_valid_pubkey = bech_key.starts_with(
                                (String::from("npub1") + cur_vanity_npub.as_str()).as_str(),
                            );

                            if is_valid_pubkey {
                                vanity_npub = cur_vanity_npub.clone();
                                break;
                            }
                        }
                    }
                    else {
                        for cur_vanity_npub in vanity_npubs_post_ts.iter() {
                            is_valid_pubkey = bech_key.ends_with(cur_vanity_npub.as_str());

                            if is_valid_pubkey {
                                vanity_npub = cur_vanity_npub.clone();
                                break;
                            }
                        }
                    }

                } else {
                    // difficulty search
                    leading_zeroes = get_leading_zero_bits(&xonly_public_key.serialize());
                    is_valid_pubkey = leading_zeroes > best_diff.load(Ordering::Relaxed);
                    if is_valid_pubkey {
                        // update difficulty only if it was set in the first place
                        if best_diff.load(Ordering::Relaxed) > 0 {
                            best_diff
                                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |_| {
                                    Some(leading_zeroes)
                                })
                                .unwrap();
                        }
                    }
                }

                // if one of the required conditions is satisfied
                if is_valid_pubkey {
                    println!("==============================================");
                    print_keys(secret_key, xonly_public_key, vanity_npub, leading_zeroes).unwrap();
                    let iterations = iterations.load(Ordering::Relaxed);
                    let iter_string = format!("{iterations}");
                    let l = iter_string.len();
                    let f = iter_string.chars().next().unwrap();
                    println!(
                        "{} iterations (about {}x10^{} hashes) in {} seconds. Avg rate {} hashes/second",
                        iterations,
                        f,
                        l - 1,
                        now.elapsed().as_secs(),
                        iterations / max(1, now.elapsed().as_secs())
                    );
                }
            }
        });
    }

    // put main thread to sleep
    loop {
        thread::sleep(std::time::Duration::from_secs(3600));
    }
}

/// Benchmark the cores capabilities for key generation
fn benchmark_cores(cores: usize, pow_difficulty: u8) {
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
fn print_keys(
    secret_key: SecretKey,
    xonly_public_key: XOnlyPublicKey,
    vanity_npub: String,
    leading_zeroes: u8,
) -> Result<(), Box<dyn Error>> {
    if leading_zeroes != 0 {
        println!("Leading zero bits:         {leading_zeroes}");
    } else if !vanity_npub.is_empty() {
        println!("Vanity npub found:         {vanity_npub}")
    }

    println!("Found matching public key: {xonly_public_key}");

    let private_hex = secret_key.display_secret().to_string();
    println!("Nostr private key: {private_hex:>72}");

    println!(
        "Nostr public key (npub): {:>65}",
        bech32::encode(
            "npub",
            hex::decode(xonly_public_key.to_hex())?.to_base32(),
            Variant::Bech32
        )?
    );
    println!(
        "Nostr private key (nsec): {:>64}",
        bech32::encode(
            "nsec",
            hex::decode(private_hex)?.to_base32(),
            Variant::Bech32
        )?
    );

    Ok(())
}

#[inline]
fn get_leading_zero_bits(bytes: &[u8]) -> u8 {
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
