use clap::Parser;

#[derive(Parser)]
#[command(author = "Rana", about = "A simple CLI to generate nostr vanity addresses")]
pub struct CLIArgs {
    /// Name of the person to greet
    #[arg(
        short,
        long,
        default_value_t = 10,
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
    #[arg(short = 'n', long = "long", required = false, default_value = "", help="Enter the prefix your public key should have when expressed
in npub format (Bech32 encoding).
This can be combined with --vanity, but beware of extra
calculations required.")]
    pub vanity_npub_prefix: String,
}



