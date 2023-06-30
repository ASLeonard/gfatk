// Max Brown
// Wellcome Sanger Institute 2023
// Modified further by Alex Leonard ETHZ 2023

use std::path::PathBuf;

use anyhow::Result;
use clap::{crate_version, value_parser, Arg, ArgAction, Command};
use gfatk::SSC;

fn main() -> Result<()> {
    let matches = Command::new("gfatk")
        .version(crate_version!())
        .propagate_version(true)
        .arg_required_else_help(true)
        .author("Max Brown <mb39@sanger.ac.uk>")
        .about("Explore and linearise (plant organellar) GFA files.")
        .subcommand(
            Command::new("SSC")
                .about("Extract Strongly Connected Components from a GFA.")
                .arg(
                    Arg::new("GFA")
                        .value_parser(value_parser!(PathBuf))
                        .help("Input GFA file.")
                )
                .arg(
                    Arg::new("Size")
                        .short('s')
                        .long("size")
                        .default_value("5")
                        .value_parser(value_parser!(usize))
                        .help("min SSC size to consider."),
                ),
        )
                

    match matches.subcommand() {
        Some(("SSC", matches)) => {
            SSC::get_strong_terminal_nodes(matches)?;
        }
        _ => {
            eprintln!("Subcommand invalid, run with '--help' for subcommand options. Exiting.");
            std::process::exit(1);
        }
    }

    Ok(())
}
