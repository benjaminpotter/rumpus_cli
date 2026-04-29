use anyhow::Result;
use clap::Parser;
use rumpus_cli::cli::Cli;
use rumpus_cli::cli::Commands;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Simulate {
            params,
            target,
            output,
            format,
        } => rumpus_cli::simulate::run(params, target, output, format),
        Commands::Parse {
            file,
            output,
            min_dop,
            target,
            format,
        } => rumpus_cli::parse::run(file, output, min_dop, target, format),
    }
}
