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
as hexadecimal."
    )]
    pub vanity_prefix: String,
    #[arg(
        short = 'n',
        long = "vanity-n-prefix",
        required = false,
        default_value = "",
        help = "Enter the prefix your public key should have when expressed
in npub format (Bech32 encoding). Specify multiple vanity
targets as a comma-separated list."
    )]
    pub vanity_npub_prefixes_raw_input: String,
    #[arg(
        short = 's',
        long = "vanity-n-suffix",
        required = false,
        default_value = "",
        help = "Enter the suffix your public key should have when expressed
in npub format (Bech32 encoding). Specify multiple vanity
targets as a comma-separated list."
    )]
    pub vanity_npub_suffixes_raw_input: String,
    #[arg(
        short = 'c',
        long = "cores",
        default_value_t = num_cpus::get(),
        help = "Number of processor cores to use"
    )]
    pub num_cores: usize,
}

pub fn check_args(difficulty: u8, vanity_prefix: &str, vanity_npub_prefixes: &Vec<String>, vanity_npub_suffixes: &Vec<String>, num_cores: usize) {
    // Check the public key requirements
    let mut requirements_count: u8 = 0;
    if difficulty > 0 {
        requirements_count += 1;
    }
    if !vanity_prefix.is_empty() {
        requirements_count += 1;
    }
    if !vanity_npub_prefixes.is_empty() || !vanity_npub_suffixes.is_empty() {
        requirements_count += 1;
    }

    if requirements_count > 1 {
        panic!("You can cannot specify more than one requirement. You should choose between difficulty or any of the vanity formats.");
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

    for vanity_npub_suffix in vanity_npub_suffixes {
        if !vanity_npub_suffix.is_empty() {
            let hex_re = Regex::new(r"^([02-9ac-hj-np-z]*)$").unwrap();
            if !hex_re.is_match(vanity_npub_suffix.as_str()) {
                panic!("The vanity npub suffix can only contain characters supported by Bech32: 023456789acdefghjklmnpqrstuvwxyz");
            }
        }
        if vanity_npub_suffix.len() > 59 {
            panic!("The vanity npub suffix cannot be longer than 59 characters.");
        }
    }

    if num_cores == 0 {
        panic!("There can be no proof of work if one does not do work (-c, --cores must be greater than 0)");
    } else if num_cores > num_cpus::get() {
        panic!("Your processor has {} cores; cannot set -c, --cores to {}", num_cpus::get(), num_cores);
    }

}
