# Rumpus CLI

A command-line interface for various skylight polarization utilities.
This crate provides an example implementation of the [rumpus](https://github.com/benjaminpotter/rumpus) library.

## Installation

### From crates.io

```bash
cargo install rumpus_cli
```

### From source

```bash
git clone https://github.com/benjaminpotter/rumpus_cli.git
cd rumpus_cli
cargo install --path .
```

## Usage

```
Usage: rumpus <COMMAND>

Commands:
  simulate  Generate a simulation of the skylight polarization pattern
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### Commands

#### Simulate

```
Generate a simulation of the skylight polarization pattern

Usage: rumpus simulate [OPTIONS] --output <OUTPUT>

Options:
  -p, --params <PARAMS>
          Optional path to TOML formatted SensorParams.
          
          If not provided, the default SensorParams is used.

  -o, --output <OUTPUT>
          File path for the simulated output

  -f, --format <FORMAT>
          Format for simulation output.
          
          If not provided, the output format is inferred from the file extension.
          
          [possible values: png, dat]

  -h, --help
          Print help (see a summary with '-h')
```

## License

This project is licensed under the GPLv3 License - see the LICENSE file for details.
