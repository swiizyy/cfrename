mod config;
mod i18n;
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
    } else if let Ok(config) = config::Config::load_default() {
        config
    } else {
        let messages = i18n::Messages::for_language(i18n::Language::default());
        eprintln!("{}", messages.error_config_not_found);
        eprintln!("{} {}", messages.error_expected_location, config::Config::default_path()?.display());
        eprintln!("\n{}", messages.error_create_config);
        std::process::exit(1);
    };

    let messages = i18n::Messages::for_language(config.language);

    if !cli.file.exists() {
        anyhow::bail!("{} {}", messages.error_file_not_exist, cli.file.display());
    }

    if !cli.file.is_file() {
        anyhow::bail!("{} {}", messages.error_not_a_file, cli.file.display());
    }

    let session = interactive::InteractiveSession::new(config.clone());
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

    // Resolve the target directory using base_path if configured
    let resolved_target = config.resolve_target_path(request.target_directory.as_deref());

    let target_path = operations::build_target_path(
        &cli.file,
        &new_filename,
        resolved_target.as_deref(),
    );

    let operation = operations::FileOperation::new(cli.file, target_path, messages);
    operation.preview();

    if operation.confirm()? {
        operation.execute()?;
    } else {
        println!("{}", messages.operation_cancelled);
    }

    Ok(())
}
