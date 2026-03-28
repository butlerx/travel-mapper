use leptos::prelude::*;

#[component]
pub(super) fn TripitSection(
    has_tripit: bool,
    sync_status: Option<String>,
    last_sync_at: Option<String>,
    trips_fetched: Option<i64>,
    hops_fetched: Option<i64>,
) -> impl IntoView {
    let has_sync = sync_status.is_some();
    view! {
        <section class="card">
            {if has_tripit {
                view! {
                    <div class="card-header-actions">
                        <h2>
                            "TripIt"
                            <span class="badge badge-success ml-sm">"Connected"</span>
                        </h2>
                        <form method="post" action="/sync">
                            <button class="btn btn-success btn-sm" type="submit">"Sync Now"</button>
                        </form>
                    </div>
                    {if has_sync {
                        view! {
                            <div class="settings-sync-grid">
                                <div class="stat-card">
                                    <div class="stat-label">"Status"</div>
                                    <div class="stat-value">{sync_status.unwrap_or_default()}</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-label">"Last Sync"</div>
                                    <div class="stat-value">{last_sync_at.unwrap_or_else(|| "\u{2014}".to_owned())}</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-label">"Trips"</div>
                                    <div class="stat-value">{trips_fetched.unwrap_or(0)}</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-label">"Journeys"</div>
                                    <div class="stat-value">{hops_fetched.unwrap_or(0)}</div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }}
                }.into_any()
            } else {
                view! {
                    <h2>
                        "TripIt"
                        <span class="badge badge-muted ml-sm">"Not Connected"</span>
                    </h2>
                    <p>"Connect your TripIt account to sync your travel history."</p>
                    <div class="form-actions">
                        <a class="btn btn-primary" href="/auth/tripit/connect">"Connect TripIt"</a>
                    </div>
                }.into_any()
            }}
        </section>
    }
}
