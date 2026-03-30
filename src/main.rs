use clap::Parser;

#[derive(Parser)]
#[command(about = "Travel Mapper — sync TripIt travel history and serve it via a web dashboard")]
struct Cli {
    #[command(subcommand)]
    command: travel_mapper::commands::Command,
}

#[tokio::main]
async fn main() {
    travel_mapper::telemetry::init();

    let cli = Cli::parse();

    if let Err(error) = travel_mapper::commands::run(cli.command).await {
        tracing::error!(%error, "fatal error");
        std::process::exit(1);
    }
}
