mod api_keys_section;
mod flighty_section;
mod sync_section;
mod tripit_section;

use super::{navbar::NavBar, shell::Shell};
use api_keys_section::ApiKeysSection;
use flighty_section::FlightySection;
use leptos::prelude::*;
use sync_section::SyncSection;
use tripit_section::TripitSection;

#[component]
pub fn SettingsPage(
    has_tripit: bool,
    sync_status: Option<String>,
    last_sync_at: Option<String>,
    trips_fetched: Option<i64>,
    hops_fetched: Option<i64>,
    #[prop(optional_no_strip)] error: Option<String>,
    #[prop(optional_no_strip)] tripit_connected: Option<String>,
    #[prop(optional_no_strip)] flighty_imported: Option<String>,
) -> impl IntoView {
    view! {
        <Shell title="Settings".to_owned()>
            <NavBar current="settings" />
            <main class="container">
                {error.map(|e| view! {
                    <div class="alert alert-error" role="alert">{e}</div>
                })}
                {tripit_connected.filter(|v| v == "connected").map(|_| view! {
                    <div class="alert alert-success" role="status">"TripIt account connected successfully!"</div>
                })}
                {flighty_imported.map(|count| view! {
                    <div class="alert alert-success" role="status">
                        {format!("Successfully imported {count} flights from Flighty!")}
                    </div>
                })}

                <TripitSection has_tripit=has_tripit />

                <SyncSection
                    has_tripit=has_tripit
                    sync_status=sync_status
                    last_sync_at=last_sync_at
                    trips_fetched=trips_fetched
                    hops_fetched=hops_fetched
                />

                <FlightySection />

                <ApiKeysSection />
            </main>
        </Shell>
    }
}
