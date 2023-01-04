use clap::{Parser};

#[derive(Parser)]
#[command(author = "Rana", about = "A simple CLI to generate vanity addresses")]
pub struct CLIArgs {
    /// Name of the person to greet
    #[arg(short, long, default_value_t = 10)]
    pub difficulty: u8,
    #[arg(short, long, required = false, default_value = "")]
    pub vanity_prefix: String,
    #[arg(short='n', long, required = false, default_value = "")]
    pub vanity_npub_prefix: String,

}