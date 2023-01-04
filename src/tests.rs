#[cfg(test)]
mod tests {
    use crate::cli::CLIArgs;

    #[test]
    fn cli_tests() {
        use clap::CommandFactory;
        CLIArgs::command().debug_assert();
    }
}