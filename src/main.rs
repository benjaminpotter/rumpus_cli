use anyhow::Result;
use anyhow::bail;
use clap::Parser;
use rumpus_cli::cli::Cli;
use rumpus_cli::cli::Commands;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command() {
        Commands::Simulate {
            params,
            output,
            format,
        } => rumpus_cli::simulate::run(params, output, format),
        _ => bail!("this command has not been implemented!"),
    }
}
