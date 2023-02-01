use std::process::exit;

use nostr::prelude::*;

use crate::cli::CLIArgs;

pub fn handle_mnemonic(parsed_args: &CLIArgs) {
    if parsed_args.word_count > 0 {
        let mut word_count = parsed_args.word_count;
        if parsed_args.word_count == 0 || parsed_args.word_count > 24 {
            word_count = 12;
        }
        let mnemonic = Keys::generate_mnemonic(word_count).expect("Couldn't not generate mnemonic");
        let keys = Keys::from_mnemonic(mnemonic.to_string(), None).expect("");

        println!(
            "Mnemonic: {}\nPublic key: {}\nPrivate key: {}",
            mnemonic,
            keys.public_key()
                .to_bech32()
                .expect("Could not get public key bech32 conversion"),
            keys.secret_key()
                .expect("Could not get secret key")
                .to_bech32()
                .expect("Could not get secret key bech32 conversion")
        );
        exit(0)
    }

    if !parsed_args.mnemonic.is_empty() {
        let keys = Keys::from_mnemonic(
            parsed_args.mnemonic.to_string(),
            Some(parsed_args.mnemonic_passphrase.to_string()),
        )
        .expect("Error creating key pair from mnemonic");

        println!(
            "Public key: {}\nPrivate key: {}",
            keys.public_key()
                .to_bech32()
                .expect("Could not get public key bech32 conversion"),
            keys.secret_key()
                .expect("Could not get secret key")
                .to_bech32()
                .expect("Could not get secret key bech32 conversion")
        );
        exit(0);
    }
}
