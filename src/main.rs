mod etw;
mod result;

use clap::{Parser, Subcommand};
use etw::{start_trace, stop_trace};
use windows::core::{Result, GUID};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Start {
        #[clap(short, long)]
        name: String,
        #[clap(short, long)]
        provider: String,
    },
    Stop {
        #[clap(short, long)]
        name: String,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Start { name, provider } => {
            stop_trace(&name)?;

            let provider_id: GUID = provider.as_str().into();
            let _handle = start_trace(&name, &provider_id)?;
        }
        Commands::Stop { name } => {
            stop_trace(&name)?;
        }
    }

    Ok(())
}
