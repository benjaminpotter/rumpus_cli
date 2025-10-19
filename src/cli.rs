use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Commands {
    /// Generate a simulation of the skylight polarization pattern.
    Simulate {
        /// Optional path to TOML formatted SensorParams.
        ///
        /// If not provided, the default SensorParams is used.
        #[arg(short, long)]
        params: Option<PathBuf>,

        /// File path for the simulated output.
        #[arg(short, long)]
        output: PathBuf,

        /// Format for simulation output.
        ///
        /// If not provided, the output format is inferred from the file extension.
        #[arg(short, long, value_enum)]
        format: Option<SimulationFormat>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[non_exhaustive]
pub enum SimulationFormat {
    Png,
    Dat,
}

impl Cli {
    pub fn command(&self) -> Commands {
        self.command.clone()
    }
}
