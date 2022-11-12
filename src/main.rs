use secp256k1::rand::rngs::OsRng;
use secp256k1::Secp256k1;
use std::cmp::max;
use std::env;
use std::error::Error;
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let difficulty = args.get(1);

    let mut pow_difficulty = 10;
    if let Some(d) = difficulty {
        pow_difficulty = d
            .trim()
            .parse()
            .expect("You must enter a number less than 255");
    }
    println!("Started mining process with a difficulty of: {pow_difficulty}");

    // Loop: generate public keys until desired number of leading zeroes is reached
    let now = Instant::now();
    let mut iterations = 0;
    let mut found_valid_hash = false;

    let secp = Secp256k1::new();

    while !found_valid_hash {
        iterations += 1;

        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        let (xonly_public_key, _) = public_key.x_only_public_key();
        let leading_zeroes = get_leading_zero_bits(&xonly_public_key.serialize());
        if leading_zeroes >= pow_difficulty {
            found_valid_hash = true;

            println!("Found matching public key: {xonly_public_key}");
            println!("Leading zero bits: {leading_zeroes} (min. required: {pow_difficulty})");
            let iter_string = format!("{iterations}");
            let l = iter_string.len();
            let f = iter_string.chars().next().unwrap();
            println!(
                "{} iterations (about {}x10^{} hashes) in {} seconds. Avg rate {} hashes/second",
                iterations,
                f,
                l - 1,
                now.elapsed().as_secs(),
                iterations * 1000 / max(1, now.elapsed().as_millis())
            );
            let private = secret_key.display_secret().to_string();
            println!("Nostr private key: {private}");
        }
    }

    Ok(())
}

#[inline]
fn get_leading_zero_bits(bytes: &[u8]) -> u8 {
    let mut res = 0_u8;
    for b in bytes.as_ref() {
        if *b == 0 {
            res += 8;
        } else {
            res += b.leading_zeros() as u8;
            return res;
        }
    }
    res
}
