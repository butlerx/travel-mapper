use super::shell::Shell;
use leptos::prelude::*;

#[component]
pub fn LandingPage() -> impl IntoView {
    view! {
        <Shell title="Home".to_owned()>
            <main class="auth-page">
                <div class="card auth-card">
                    <div class="hero">
                        <h1>"Travel Export"</h1>
                        <p>"Sync your TripIt travel history to a local database and explore it via JSON, CSV, or HTML."</p>
                        <div class="hero-actions">
                            <a class="btn btn-primary" href="/register">"Get Started"</a>
                            <a class="btn btn-secondary" href="/login">"Log In"</a>
                        </div>
                    </div>
                </div>
            </main>
        </Shell>
    }
}
