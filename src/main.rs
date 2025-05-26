use clap::{Parser, Subcommand};
use color_eyre::eyre;
use std::path::PathBuf;
use wabznasm::repl;

#[derive(Parser)]
#[command(name = "wabznasm")]
#[command(about = "A Q/KDB+ inspired array processing language")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start Jupyter kernel
    Jupyter {
        #[command(subcommand)]
        action: JupyterCommands,
    },
}

#[derive(Subcommand)]
enum JupyterCommands {
    /// Start the Jupyter kernel with a connection file
    Start {
        /// Path to the Jupyter connection file
        connection_file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), eyre::Report> {
    // Set up colorful error reporting
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Jupyter { action }) => match action {
            JupyterCommands::Start { connection_file } => {
                // Start Jupyter kernel
                use wabznasm::jupyter::kernel::JupyterKernelRunner;
                let mut kernel = JupyterKernelRunner::from_file(&connection_file)
                    .map_err(|e| eyre::eyre!("Failed to create kernel: {}", e))?;
                kernel
                    .run()
                    .await
                    .map_err(|e| eyre::eyre!("Kernel execution failed: {}", e))?;
                Ok(())
            }
        },
        None => {
            // Default to REPL
            repl::run()
        }
    }
}
