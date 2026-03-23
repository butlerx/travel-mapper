use leptos::prelude::*;

#[component]
pub(super) fn SyncSection(
    has_tripit: bool,
    sync_status: Option<String>,
    last_sync_at: Option<String>,
    trips_fetched: Option<i64>,
    hops_fetched: Option<i64>,
) -> impl IntoView {
    let has_sync = sync_status.is_some();
    view! {
        <section class="card">
            <h2>"Sync Status"</h2>
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
    }
}
