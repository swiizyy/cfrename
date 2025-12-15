mod config;
mod interactive;
mod naming;
mod operations;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cfrename")]
#[command(about = "CLI tool for standardizing file renaming and organization", long_about = None)]
struct Cli {
    #[arg(help = "Path to the file to rename")]
    file: PathBuf,

    #[arg(short, long, help = "Path to configuration file")]
    config: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = if let Some(config_path) = cli.config {
        config::Config::load(&config_path)?
    } else {
        match config::Config::load_default() {
            Ok(config) => config,
            Err(_) => {
                eprintln!("Error: Configuration file not found.");
                eprintln!("Expected location: {}", config::Config::default_path()?.display());
                eprintln!("\nPlease create a configuration file. See the example configuration.");
                std::process::exit(1);
            }
        }
    };

    if !cli.file.exists() {
        anyhow::bail!("File does not exist: {}", cli.file.display());
    }

    if !cli.file.is_file() {
        anyhow::bail!("Path is not a file: {}", cli.file.display());
    }

    let session = interactive::InteractiveSession::new(config);
    let request = session.run()?;

    let extension = naming::extract_extension(&cli.file);
    let filename_builder = naming::FileNameBuilder::new(
        request.date,
        request.doc_type,
        request.entity,
        request.description,
        extension,
    );

    let new_filename = filename_builder.build();
    let target_path = operations::build_target_path(
        &cli.file,
        &new_filename,
        request.target_directory.as_deref(),
    );

    let operation = operations::FileOperation::new(cli.file, target_path);
    operation.preview();

    if operation.confirm()? {
        operation.execute()?;
    } else {
        println!("Operation cancelled.");
    }

    Ok(())
}
