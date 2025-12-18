mod config;
mod i18n;
mod interactive;
mod naming;
mod operations;
mod path_validation;

use anyhow::Result;
use clap::Parser;
use dialoguer::Confirm;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cfrename")]
#[command(version)]
#[command(about = "CLI tool for standardizing file renaming and organization", long_about = None)]
struct Cli {
    #[arg(help = "Path to the file to rename")]
    file: PathBuf,

    #[arg(short, long, help = "Path to configuration file")]
    config: Option<PathBuf>,

    #[arg(long, help = "Allow following symbolic links (disabled by default for security)")]
    follow_symlinks: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = if let Some(config_path) = cli.config {
        config::Config::load(&config_path)?
    } else if let Ok(config) = config::Config::load_default() {
        config
    } else {
        // Config not found - prompt user to create default config
        let messages = i18n::Messages::for_language(i18n::Language::default());
        eprintln!("{}", messages.error_config_not_found);
        eprintln!("{} {}", messages.error_expected_location, config::Config::default_path()?.display());
        eprintln!();

        let should_create = Confirm::new()
            .with_prompt(messages.prompt_create_default_config)
            .default(true)
            .interact()?;

        if should_create {
            let created_path = config::Config::create_default_config()?;
            println!("{}", messages.success_config_created);
            println!("Location: {}", created_path.display());
            println!("\nPlease review and customize the configuration file, then run cfrename again.");
            std::process::exit(0);
        } else {
            eprintln!("\n{}", messages.error_create_config);
            std::process::exit(1);
        }
    };

    let messages = i18n::Messages::for_language(config.language);

    // Note: File validation (exists, is_file, symlink detection) is now performed
    // by PathValidator in FileOperation::new()

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

    let operation = operations::FileOperation::new(
        cli.file,
        target_path,
        messages,
        cli.follow_symlinks,
    )?;
    operation.preview();

    if operation.confirm()? {
        operation.execute()?;
    } else {
        println!("{}", messages.operation_cancelled);
    }

    Ok(())
}
