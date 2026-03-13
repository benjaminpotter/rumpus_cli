use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone, PartialEq, PartialOrd)]
pub enum Commands {
    /// Generate a simulation of the skylight polarization pattern.
    Simulate {
        /// Optional path to TOML formatted [`SensorParams`].
        ///
        /// If not provided, the default [`SensorParams`] is used.
        #[arg(short, long)]
        params: Option<PathBuf>,

        /// Simulation target.
        #[arg(short, long, value_enum)]
        target: Target,

        /// File path for the simulated output.
        #[arg(short, long)]
        output: PathBuf,

        /// Format for simulation output.
        ///
        /// If not provided, the output format is inferred from the file extension.
        #[arg(short, long, value_enum)]
        format: Option<Format>,
    },

    /// Parse an intensity image into target polarization data.
    Parse {
        file: PathBuf,

        #[arg(short, long, default_value = "out.png")]
        output: PathBuf,

        #[arg(short, long, default_value_t = 0.0)]
        min_dop: f64,

        #[arg(short, long, value_enum)]
        target: Target,

        #[arg(short, long, value_enum)]
        format: Option<Format>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[non_exhaustive]
pub enum Target {
    AopSensor,
    AopGlobal,
    Dop,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[non_exhaustive]
pub enum Format {
    Png,
    Dat,
    Bin,
}
