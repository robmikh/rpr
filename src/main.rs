mod etw;
mod guid;
mod providers;

use std::path::Path;

use clap::{Parser, Subcommand};
use etw::{start_trace, stop_trace};
use guid::TryParse;
use providers::enumerate_providers;
use wildmatch::WildMatch;
use windows::core::{GUID, Result};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Starts a trace with a given provider
    Start {
        #[clap(short, long)]
        /// Name of the session
        name: String,
        #[clap(short, long)]
        /// Output file (defaults to "<NAME>.etl")
        file: Option<String>,
        /// ETW provider to enable and trace. Can be either a GUID or exact provider name.
        #[clap(short, long)]
        provider: String,
    },
    /// Stops an ongoing trace session
    Stop {
        /// Name of the session to stop
        #[clap(short, long)]
        name: String,
    },
    /// Lists the registered providers on the system
    List {
        /// Search string. Supports wildcard matches.
        #[clap(short, long)]
        search: Option<String>,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Start {
            name,
            file,
            provider,
        } => {
            // Stop the previous session if it exists
            if stop_trace(&name)? {
                println!("Stopped previous session.");
            }

            // Validate the file path and create any folders
            let file = if let Some(file) = file {
                if !validate_path(&file) {
                    exit_with_error("Invalid file specified!");
                }
                ensure_path(&file);
                file
            } else {
                format!("{}.etl", name)
            };

            // Start the tracing session
            let provider_str = provider.as_str();
            let provider_id: Option<GUID> = if let Some(provider) = provider_str.try_parse() {
                Some(provider)
            } else {
                let providers = enumerate_providers()?;
                let provider_info = providers.iter().find(|x| x.name == provider_str);
                if let Some(provider_info) = provider_info {
                    Some(provider_info.guid)
                } else {
                    println!("No provider with name \"{}\" found.", provider_str);
                    None
                }
            };
            if let Some(provider_id) = provider_id {
                let _handle = start_trace(&name, &file, &provider_id)?;
                println!("Trace started.");
            }
        }
        Commands::Stop { name } => {
            if !stop_trace(&name)? {
                println!("No session with name \"{}\" found.", name);
            } else {
                println!("Trace stopped.");
            }
        }
        Commands::List { search } => {
            let providers = enumerate_providers()?;
            let wildmatch = if let Some(search) = search {
                Some(WildMatch::new(&search.to_lowercase()))
            } else {
                None
            };
            for provider in providers {
                if let Some(wildmatch) = wildmatch.as_ref() {
                    if !wildmatch.matches(&provider.name.to_lowercase()) {
                        continue;
                    }
                }
                println!("{} - {:?}", provider.name, provider.guid);
            }
        }
    }

    Ok(())
}

fn validate_path<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    let mut valid = true;
    if let Some(extension) = path.extension() {
        if extension != "etl" {
            valid = false;
        }
    } else {
        valid = false;
    }
    valid
}

fn ensure_path<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    if let Some(parent_path) = path.parent() {
        std::fs::create_dir_all(parent_path).unwrap();
    }
}

fn exit_with_error(message: &str) -> ! {
    println!("{}", message);
    std::process::exit(1);
}
