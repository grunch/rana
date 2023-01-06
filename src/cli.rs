use clap::Parser;
use regex::Regex;

#[derive(Parser)]
#[command(
    name = "Rana",
    about = "A simple CLI to generate nostr vanity addresses",
    author,
    help_template = "\
{before-help}{name} 🐸

{about-with-newline}
{author-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
",
    version
)]
pub struct CLIArgs {
    #[arg(
        short,
        long,
        default_value_t = 0,
        help = "Enter the number of starting bits that should be 0."
    )]
    pub difficulty: u8,
    #[arg(
        short,
        long = "vanity",
        required = false,
        default_value = "",
        help = "Enter the prefix your public key should have when expressed
as hexadecimal. This can be combined with --vanity-n,
but beware of extra calculations required."
    )]
    pub vanity_prefix: String,
    #[arg(
        short = 'n',
        long = "vanity-n",
        required = false,
        default_value = "",
        help = "Enter the prefix your public key should have when expressed
in npub format (Bech32 encoding). Specify multiple vanity
targets as a comma-separated list.
This can be combined with --vanity, but beware of extra
calculations required."
    )]
    pub vanity_npub_prefixes_raw_input: String,
}

pub fn check_args(difficulty: u8, vanity_prefix: &str, vanity_npub_prefixes: &Vec<String>) {
    if difficulty > 0 && (!vanity_prefix.is_empty() || !vanity_npub_prefixes.is_empty()) {
        panic!("You can cannot specify difficulty and vanity at the same time.");
    }
    if vanity_prefix.len() > 64 {
        panic!("The vanity prefix cannot be longer than 64 characters.");
    }

    if !vanity_prefix.is_empty() {
        // check valid hexa characters
        let hex_re = Regex::new(r"^([0-9a-f]*)$").unwrap();
        if !hex_re.is_match(vanity_prefix) {
            panic!("The vanity prefix can only contain hexadecimal characters.");
        }
    }

    for vanity_npub_prefix in vanity_npub_prefixes {
        if !vanity_npub_prefix.is_empty() {
            let hex_re = Regex::new(r"^([02-9ac-hj-np-z]*)$").unwrap();
            if !hex_re.is_match(vanity_npub_prefix.as_str()) {
                panic!("The vanity npub prefix can only contain characters supported by Bech32: 023456789acdefghjklmnpqrstuvwxyz");
            }
        }
        if vanity_npub_prefix.len() > 59 {
            panic!("The vanity npub prefix cannot be longer than 59 characters.");
        }
    }

}
