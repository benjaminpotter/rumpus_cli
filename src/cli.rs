use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Commands {
    /// Generate a simulation of the skylight polarization pattern.
    Simulate {
        /// Optional path to TOML formatted SensorParams.
        ///
        /// If not provided, the default SensorParams is used.
        #[arg(short, long)]
        params: Option<PathBuf>,

        /// Simulation target.
        #[arg(short, long, value_enum)]
        target: SimulationTarget,

        /// File path for the simulated output.
        #[arg(short, long)]
        output: PathBuf,

        /// Format for simulation output.
        ///
        /// If not provided, the output format is inferred from the file extension.
        #[arg(short, long, value_enum)]
        format: Option<SimulationFormat>,
    },

    /// Parse an intensity image into target polarization data.
    Parse {
        #[arg(short, long)]
        image: PathBuf,

        #[arg(short, long, default_value = "out.png")]
        output_path: PathBuf,

        #[arg(short, long, default_value_t = 6.9)]
        pixel_size_um: f64,

        #[arg(short, long, default_value_t = 0.0)]
        min_dop: f64,

        #[arg(short, long, value_enum)]
        target: ParseTarget,

        #[arg(short, long, value_enum)]
        format: ParseFormat,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[non_exhaustive]
pub enum SimulationFormat {
    Png,
    Dat,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[non_exhaustive]
pub enum SimulationTarget {
    Aop,
    Dop,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[non_exhaustive]
pub enum ParseFormat {
    Png,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[non_exhaustive]
pub enum ParseTarget {
    Aop,
    Dop,
}

impl Cli {
    pub fn command(&self) -> Commands {
        self.command.clone()
    }
}
