use bech32::{ToBase32, Variant};
use bip39::Mnemonic;
use bitcoin_hashes::hex::ToHex;
use clap::Parser;

use nostr_sdk::prelude::{FromMnemonic, GenerateMnemonic, Keys};
use rana::cli::*;
use rana::mnemonic::handle_mnemonic;
use rana::utils::{benchmark_cores, get_leading_zero_bits, print_keys};
use secp256k1::rand::thread_rng;
use secp256k1::Secp256k1;
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

    // Handle mnemonic part if arguments is set
    if parsed_args.vanity_prefix.len() == 0
        && parsed_args.vanity_npub_prefixes_raw_input.len() == 0
        && parsed_args.difficulty == 0
    {
        handle_mnemonic(&parsed_args);
    }

    let mut difficulty = parsed_args.difficulty;
    let vanity_prefix = parsed_args.vanity_prefix;
    let mut vanity_npub_prefixes = <Vec<String>>::new();
    let num_cores = parsed_args.num_cores;

    for vanity_npub in parsed_args.vanity_npub_prefixes_raw_input.split(',') {
        if !vanity_npub.is_empty() {
            vanity_npub_prefixes.push(vanity_npub.to_string())
        }
    }

    //-- Calculate pow difficulty and initialize
    check_args(
        difficulty,
        vanity_prefix.as_str(),
        &vanity_npub_prefixes,
        num_cores,
    );

    
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
    if !vanity_npub_prefixes.is_empty() {
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
    let vanity_npubs_ts = Arc::new(vanity_npub_prefixes);
    let iterations = Arc::new(AtomicU64::new(0));

    // start a thread for each core for calculations
    for _ in 0..num_cores {
        let best_diff = best_diff.clone();
        let vanity_ts = vanity_ts.clone();
        let vanity_npubs_ts = vanity_npubs_ts.clone();
        let iterations = iterations.clone();
        thread::spawn(move || {
            let mut rng = thread_rng();
            let secp = Secp256k1::new();

            let mut keys;
            let mut mnemonic;
            let mut xonly_pub_key;

            loop {
                let mut uses_mnemonic: Option<Mnemonic> = None;
                iterations.fetch_add(1, Ordering::Relaxed);

                let secret_key_string: String;
                let xonly_public_key_serialized;
                let hexa_key;

                // Use mnemonics to generate key pair
                if parsed_args.word_count > 0 {
                    mnemonic = Keys::generate_mnemonic(parsed_args.word_count)
                        .expect("Couldn't not generate mnemonic");

                    uses_mnemonic = Some(mnemonic.clone()).clone();
                    keys = Keys::from_mnemonic(mnemonic.to_string(), None).expect("");
                    hexa_key = keys.public_key().to_hex();
                    xonly_pub_key = hexa_key.clone();
                    secret_key_string = keys
                        .secret_key()
                        .expect("Couldn't get secret key")
                        .display_secret()
                        .to_string();
                    let xonly_public_key = keys.public_key();
                    xonly_public_key_serialized = xonly_public_key.serialize()
                } else {
                    // Use SECP to generate key pair
                    let (secret_key, public_key) = secp.generate_keypair(&mut rng);
                    let (xonly_public_key, _) = public_key.x_only_public_key();
                    hexa_key = xonly_public_key.to_hex();
                    secret_key_string = secret_key.display_secret().to_string();

                    let (xonly_public_key, _) = public_key.x_only_public_key();
                    xonly_public_key_serialized = xonly_public_key.serialize();
                    xonly_pub_key = hexa_key.to_string();
                }

                let mut leading_zeroes = 0;
                let mut vanity_npub = "".to_string();

                // check pubkey validity depending on arg settings
                let mut is_valid_pubkey: bool = false;

                if vanity_ts.as_str() != "" {
                    // hex vanity search
                    is_valid_pubkey = hexa_key.starts_with(vanity_ts.as_str());
                } else if !vanity_npubs_ts.is_empty() {
                    // bech32 vanity search
                    let bech_key: String = bech32::encode(
                        "npub",
                        hex::decode(hexa_key).unwrap().to_base32(),
                        Variant::Bech32,
                    )
                    .unwrap();

                    for cur_vanity_npub in vanity_npubs_ts.iter() {
                        is_valid_pubkey = bech_key.starts_with(
                            (String::from("npub1") + cur_vanity_npub.as_str()).as_str(),
                        );

                        if is_valid_pubkey {
                            vanity_npub = cur_vanity_npub.clone();
                            break;
                        }
                    }
                } else {
                    // difficulty search
                    leading_zeroes = get_leading_zero_bits(&xonly_public_key_serialized);
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

                
                let mut mnemonic_str = None;
                match uses_mnemonic {
                    Some(mnemonic_obj) => {
                        mnemonic_str = Some(mnemonic_obj.to_string());
                    }
                    None => {}
                }

                // if one of the required conditions is satisfied
                if is_valid_pubkey {
                    println!("==============================================");
                    print_keys(
                        secret_key_string,
                        xonly_pub_key.clone(),
                        vanity_npub,
                        leading_zeroes,
                        mnemonic_str,
                    )
                    .unwrap();
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
