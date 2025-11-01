use anyhow::anyhow;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use my_axum::config::{cmd::runbook, setting::Setting};

#[derive(Debug, Parser)]
#[command(
    name = "runbook",
    about = "Run operational scripts against the application",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// List all available runbooks.
    List,
    /// Run a single runbook with its raw trailing arguments.
    Run {
        name: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv();
    let setting = Setting::new();
    let cli = Cli::parse();

    match cli.command {
        Commands::List => {
            println!("Available runbooks:");
            for runbook in runbook::list() {
                println!("  {} - {}", runbook.name, runbook.description);
                println!("    {}", runbook.usage);
            }
            Ok(())
        }
        Commands::Run { name, args } => {
            let result = runbook::run(&setting, &name, &args)
                .await
                .map_err(|err| anyhow!("{err}"))?;
            println!("{}", result.message);
            Ok(())
        }
    }
}
