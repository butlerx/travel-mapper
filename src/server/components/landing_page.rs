use super::shell::Shell;
use leptos::prelude::*;

#[component]
pub fn LandingPage() -> impl IntoView {
    view! {
        <Shell title="Travel Mapper".to_owned()>
            <main class="auth-page">
                <div class="card auth-card">
                    <div class="hero">
                        <img class="hero-logo" src="/static/logo.svg" alt="" width="32" height="32" />
                        <div class="hero-badge">{"\u{1F30D}"}" Every journey. Every route. One map."</div>
                        <h1>"Your Travel Story,\u{2003}Visualised"</h1>
                        <p>"Import your trips from TripIt and see every journey mapped, measured, and beautifully presented \u{2014} air, rail, boat trips, and more."</p>
                        <div class="hero-actions">
                            <a class="btn btn-primary" href="/register">"Get Started"</a>
                            <a class="btn btn-secondary" href="/login">"Log In"</a>
                        </div>
                    </div>
                </div>

                <div class="landing-features">
                    <div class="feature-card">
                        <div class="feature-icon">{"\u{1F5FA}\u{FE0F}"}</div>
                        <h3>"Interactive Map"</h3>
                        <p>"See every route on a dark, beautiful map with frequency-weighted arcs. Filter by year or travel type."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon">{"\u{1F4CA}"}</div>
                        <h3>"Travel Stats"</h3>
                        <p>"Total distance, places visited, countries explored \u{2014} your travel history at a glance."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon">{"\u{1F504}"}</div>
                        <h3>"TripIt Sync"</h3>
                        <p>"One-click import from TripIt. Background sync keeps your data up to date automatically."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon">{"\u{1F4E4}"}</div>
                        <h3>"Export Anywhere"</h3>
                        <p>"JSON, CSV, or HTML. Feed your data into Kepler.gl, spreadsheets, or your own tools."</p>
                    </div>
                </div>
            </main>
        </Shell>
    }
}
