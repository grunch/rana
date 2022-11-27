use secp256k1::rand::thread_rng;
use secp256k1::{Secp256k1, SecretKey, XOnlyPublicKey};
use std::cmp::max;
use std::env;
use std::error::Error;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use bech32::{ToBase32, Variant};
use bitcoin_hashes::hex::ToHex;

fn main() -> Result<(), Box<dyn Error>> {
    // Parse CLI arguments
    // let args: Vec<String> = env::args().collect();
    let parsed_args = parse_args();

    let mut difficulty = parsed_args.difficulty;
    let vanity_prefix = parsed_args.vanity_prefix;
    // initially the same as difficulty
    let mut pow_difficulty = difficulty;

    if vanity_prefix != "" {
        // set pow difficulty as the lenght of the prefix
        pow_difficulty = vanity_prefix.len() as u8;
        println!(
            "Started mining process for a vanify prefix of: {} (pow: {})",
            vanity_prefix, pow_difficulty
        );
    } else {
        if difficulty == 0 {
            difficulty = 10; // default
            pow_difficulty = difficulty;
        }

        println!(
            "Started mining process with a difficulty of: {difficulty} (pow: {})",
            pow_difficulty
        );
    }

    let cores = num_cpus::get();
    let mut hashes_per_second_per_core = 0;

    println!("Benchmarking a single core for 5 seconds...");
    let now = Instant::now();
    let secp = Secp256k1::new();
    let mut rng = thread_rng();
    loop {
        let (_secret_key, public_key) = secp.generate_keypair(&mut rng);
        let (xonly_public_key, _) = public_key.x_only_public_key();
        let _leading_zeroes = get_leading_zero_bits(&xonly_public_key.serialize());
        hashes_per_second_per_core += 1;
        if now.elapsed().as_secs() > 5 {
            break;
        }
    }
    hashes_per_second_per_core /= 10;
    println!("A single core can mine roughly {hashes_per_second_per_core} h/s!");

    let estimated_hashes = 2_u128.pow(pow_difficulty as u32);
    println!("Searching for prefix of {pow_difficulty} zero bits");
    let estimate = estimated_hashes as f32 / hashes_per_second_per_core as f32 / cores as f32;
    println!("This is estimated to take about {estimate} seconds");

    // Loop: generate public keys until desired number of leading zeroes is reached
    let now = Instant::now();

    println!("Mining using {cores} cores...");

    // thread safe variables
    let best_diff = Arc::new(AtomicU8::new(pow_difficulty));
    let vanity_ts = Arc::new(vanity_prefix);
    let iterations = Arc::new(AtomicU64::new(0));

    for _ in 0..cores {
        let best_diff = best_diff.clone();
        let vanity_ts = vanity_ts.clone();
        let iterations = iterations.clone();
        thread::spawn(move || {
            let mut rng = thread_rng();
            let secp = Secp256k1::new();
            loop {
                iterations.fetch_add(1, Ordering::Relaxed);

                let (secret_key, public_key) = secp.generate_keypair(&mut rng);
                let (xonly_public_key, _) = public_key.x_only_public_key();

                let leading_zeroes = get_leading_zero_bits(&xonly_public_key.serialize());

                let is_valid_pubkey: bool;
                if vanity_ts.as_str() != "" {
                    is_valid_pubkey = xonly_public_key.to_hex().starts_with(vanity_ts.as_str());
                } else {
                    is_valid_pubkey = leading_zeroes > best_diff.load(Ordering::Relaxed);
                    if is_valid_pubkey {
                        println!("Leading zero bits: {leading_zeroes}");
                    }
                }

                // if one of the required conditions is satisfied
                if is_valid_pubkey {
                    println!("==============================================");
                    print_keys(secret_key, xonly_public_key).unwrap();
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

                    best_diff.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |_| {
                        Some(leading_zeroes)
                    })
                    .unwrap();
                }
            }
        });
    }

    // put main thread to sleep
    loop {
        thread::sleep(std::time::Duration::from_secs(3600));
    }
}

/// Print private and public keys to the output
fn print_keys(
    secret_key: SecretKey,
    xonly_public_key: XOnlyPublicKey,
) -> Result<(), Box<dyn Error>> {
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

struct CliParsedArgs {
    difficulty: u8,
    vanity_prefix: String,
}

/// Parse and structure the CLI arguments
fn parse_args() -> CliParsedArgs {
    let mut parsed_args = CliParsedArgs {
        difficulty: 0,                 // empty/disabled
        vanity_prefix: "".to_string(), // empty/disabled
    };
    let args: Vec<String> = env::args().collect();

    for a in 0..args.len() {
        let arg = &args[a];
        // if named arg
        if arg.starts_with("--") {
            let arg_parts: Vec<&str> = arg.split("=").collect();
            let arg_name = (&arg_parts[0]).to_string();
            // remove the first "--"
            let arg_name = &arg_name[2..arg_name.len()];
            let arg_value = (&arg_parts[1]).to_string();
            // now parse to the supported args
            match arg_name {
                "difficulty" => parsed_args.difficulty = arg_value.parse().unwrap(),
                "vanity" => parsed_args.vanity_prefix = arg_value,
                _ => println!("Argument '{arg_name}' not supported. Ignored"),
            }
        }
    }

    // validation
    if parsed_args.difficulty > 0 && parsed_args.vanity_prefix != "" {
        panic!("You cannot set a difficulty and a vanity prefix at the same time.");
    }
    if parsed_args.vanity_prefix.len() > 255 {
        panic!("The vanity prefix cannot be longer than 255 characters.");
    }

    // println!("Diff: {}", cli_args.difficulty);
    // println!("Vanity: {}", cli_args.vanity_prefix);
    return parsed_args;
}
