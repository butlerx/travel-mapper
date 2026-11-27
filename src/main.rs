use clap::Parser;

#[derive(Parser)]
#[command(about = "Travel Mapper — sync TripIt travel history and serve it via a web dashboard")]
struct Cli {
    #[command(subcommand)]
    command: travel_mapper::commands::Command,
}

#[tokio::main]
async fn main() {
    let tracer_provider = match travel_mapper::telemetry::init() {
        Ok(provider) => provider,
        Err(error) => {
            eprintln!("failed to initialize OpenTelemetry tracing: {error}");
            std::process::exit(1);
        }
    };

    let cli = Cli::parse();

    if let Err(error) = travel_mapper::commands::run(cli.command).await {
        tracing::error!(%error, "fatal error");
        let _ = tracer_provider.shutdown();
        std::process::exit(1);
    }
    let _ = tracer_provider.shutdown();
}
