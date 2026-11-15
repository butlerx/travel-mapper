use clap::Subcommand;

/// Interactive user creation subcommand.
pub mod create_user;
/// Detect and fix incorrect station addresses in `TripIt` source data.
pub mod fix_tripit;
/// Database seeding subcommand — only available in debug builds.
#[cfg(debug_assertions)]
pub mod seed;
/// Web server subcommand — serves the dashboard, API, and static assets.
pub mod serve;
/// Background sync worker subcommand — polls for pending sync jobs.
pub mod worker;

#[derive(Subcommand)]
pub enum Command {
    /// Run the web server.
    Serve(serve::Args),
    /// Run the background sync worker.
    Worker(worker::Args),
    /// Create a new user interactively.
    CreateUser(create_user::Args),
    /// Detect and fix incorrect addresses in `TripIt` trip data.
    FixTripit(fix_tripit::Args),
    /// Seed the database with test data for local development.
    #[cfg(debug_assertions)]
    Seed(seed::Args),
}

/// Dispatch the CLI subcommand to the appropriate handler.
///
/// # Errors
///
/// Returns an error if the subcommand fails.
pub async fn run(command: Command) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::Serve(args) => serve::run(args).await.map_err(Into::into),
        Command::Worker(args) => worker::run(args).await.map_err(Into::into),
        Command::CreateUser(args) => create_user::run(args).await.map_err(Into::into),
        Command::FixTripit(args) => fix_tripit::run(args).await.map_err(Into::into),
        #[cfg(debug_assertions)]
        Command::Seed(args) => seed::run(args).await.map_err(Into::into),
    }
}
