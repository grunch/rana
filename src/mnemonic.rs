use std::process::exit;

use crate::cli::CLIArgs;
use bech32::{ToBase32, Variant};
use bitcoin_hashes::hex::ToHex;
use nostr_sdk::prelude::{FromMnemonic, GenerateMnemonic, Keys, ToBech32};

pub fn handle_mnemonic(parsed_args: &CLIArgs) {
    if parsed_args.word_count > 0 {
        let mut word_count = parsed_args.word_count;
        if parsed_args.word_count <= 0 || parsed_args.word_count > 24 {
            word_count = 12;
        }
        let mnemonic = Keys::generate_mnemonic(word_count).expect("Couldn't not generate mnemonic");
        let keys = Keys::from_mnemonic(mnemonic.to_string(), None).expect("");

        let pub_key_as_hex =
            hex::decode(keys.public_key().to_hex()).expect("Error decoding public key to hex");
        let nostr_pubkey = bech32::encode("npub", pub_key_as_hex.to_base32(), Variant::Bech32)
            .expect("Error encoding to bech32");

        println!(
            "Mnemonic: {}\nPublic key: {}\nPrivate key: {}",
            mnemonic,
            nostr_pubkey,
            keys.secret_key()
                .expect("Could not get secret key")
                .to_bech32()
                .expect("Could not get secret key bech32 conversion")
        );
        exit(0)
    }

    if parsed_args.mnemonic.len() > 0 {
        let keys = Keys::from_mnemonic(
            parsed_args.mnemonic.to_string(),
            Some(parsed_args.mnemonic_passphrase.to_string()),
        )
        .expect("Error creating key pair from mnemonic");

        let pub_key_as_hex =
            hex::decode(keys.public_key().to_hex()).expect("Error decoding public key to hex");
        let nostr_pubkey = bech32::encode("npub", pub_key_as_hex.to_base32(), Variant::Bech32)
            .expect("Error encoding to bech32");

        println!(
            "Public key: {}\nPrivate key: {}",
            nostr_pubkey,
            keys.secret_key()
                .expect("Could not get secret key")
                .to_bech32()
                .expect("Could not get secret key bech32 conversion")
        );
        exit(0);
    }
}
