use leptos::prelude::*;

#[component]
pub fn ApiKeysSection() -> impl IntoView {
    view! {
        <section class="card">
            <h2>"API Keys"</h2>
            <p>"Use API keys for programmatic access to your travel data."</p>
            <div class="mt-sm">
                <code>"POST /auth/api-keys"</code>
            </div>
            <p class="mt-sm">"Create keys via the API with a session cookie or existing API key."</p>
        </section>
    }
}
