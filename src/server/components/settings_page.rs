use super::{navbar::NavBar, shell::Shell};
use leptos::prelude::*;

#[component]
pub fn SettingsPage(
    has_tripit: bool,
    sync_status: Option<String>,
    last_sync_at: Option<String>,
    trips_fetched: Option<i64>,
    hops_fetched: Option<i64>,
    #[prop(optional_no_strip)] error: Option<String>,
    #[prop(optional_no_strip)] tripit_connected: Option<String>,
) -> impl IntoView {
    let has_sync = sync_status.is_some();
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

                <section class="card">
                    <h2>"TripIt Connection"</h2>
                    {if has_tripit {
                        view! { <span class="status-badge status-connected">"Connected"</span> }.into_any()
                    } else {
                        view! {
                            <span class="status-badge status-disconnected">"Not Connected"</span>
                            <div class="mt-md">
                                <a class="btn btn-primary" href="/auth/tripit/connect">"Connect TripIt"</a>
                            </div>
                        }.into_any()
                    }}
                </section>

                <section class="card">
                    <h2>"Sync Status"</h2>
                    {if has_sync {
                        view! {
                            <div class="stat-row">
                                <div class="stat-card">
                                    <div class="stat-label">"Status"</div>
                                    <div class="stat-value">{sync_status.unwrap_or_default()}</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-label">"Last Sync"</div>
                                    <div class="stat-value">{last_sync_at.unwrap_or_else(|| "never".to_owned())}</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-label">"Trips"</div>
                                    <div class="stat-value">{trips_fetched.unwrap_or(0)}</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-label">"Hops"</div>
                                    <div class="stat-value">{hops_fetched.unwrap_or(0)}</div>
                                </div>
                            </div>
                            {has_tripit.then(|| view! {
                                <form class="mt-md" method="post" action="/sync">
                                    <button class="btn btn-success" type="submit">"Sync Now"</button>
                                </form>
                            })}
                        }.into_any()
                    } else {
                        view! {
                            <div class="empty-state">
                                <div class="empty-state-icon">"~"</div>
                                <p>"No sync data yet."</p>
                            </div>
                        }.into_any()
                    }}
                </section>

                <section class="card">
                    <h2>"API Keys"</h2>
                    <p>"Use API keys for programmatic access to your travel data."</p>
                    <div class="mt-sm">
                        <code>"POST /auth/api-keys"</code>
                    </div>
                    <p class="mt-sm">"Create keys via the API with a session cookie or existing API key."</p>
                </section>
            </main>
        </Shell>
    }
}
