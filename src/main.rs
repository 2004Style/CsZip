//! CsZip - Herramienta de línea de comandos
//!
//! Uso:
//!   cszip compress <input> [-o output] [-a algorithm] [-l level]
//!   cszip decompress <input> [-o output]
//!   cszip info <input>
//!   cszip verify <input>

use clap::Parser;

use cszip::cli::args::{Cli, Commands};
use cszip::cli::commands;
use cszip::error::Error;

fn main() {
    let cli = Cli::parse();
    let _verbosity = cli.verbosity();

    let result = run(cli);

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        if let Some(ctx) = e.context() {
            eprintln!("  Contexto: {}", ctx);
        }
        std::process::exit(e.code() as i32);
    }
}

fn run(cli: Cli) -> Result<(), Error> {
    let verbosity = cli.verbosity();

    match cli.command {
        Commands::Compress {
            input,
            output,
            algorithm,
            level,
            force,
            crc64,
        } => commands::compress(
            &input,
            output.as_deref(),
            algorithm,
            level,
            force,
            crc64,
            verbosity,
        ),

        Commands::Decompress {
            input,
            output,
            force,
            no_verify,
        } => commands::decompress(&input, output.as_deref(), force, no_verify, verbosity),

        Commands::Info { input, detailed } => commands::info(&input, detailed, verbosity),

        Commands::Verify { input } => commands::verify(&input, verbosity),

        Commands::List { input } => commands::list(&input, verbosity),
    }
}
