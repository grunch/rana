use std::process::exit;

use crate::cli::CLIArgs;
use nostr_sdk::prelude::{FromMnemonic, GenerateMnemonic, Keys, ToBech32};

pub fn handle_mnemonic(parsed_args: &CLIArgs) {
    if parsed_args.word_count > 0 {
        let mut word_count = parsed_args.word_count;
        if parsed_args.word_count <= 0 || parsed_args.word_count > 24 {
            word_count = 12;
        }
        println!(
            "Mnemonic: {}",
            Keys::generate_mnemonic(word_count).expect("")
        );
        exit(0)
    }

    if parsed_args.mnemonic.len() > 0 {
        let keys = Keys::from_mnemonic(
            parsed_args.mnemonic.to_string(),
            Some(parsed_args.mnemonic_passphrase.to_string()),
        )
        .expect("");
        println!(
            "Public key: {}\nPrivate key: {}",
            keys.public_key(),
            keys.secret_key().expect("").to_bech32().expect("")
        );
        exit(0);
    }
}
