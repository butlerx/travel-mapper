use leptos::prelude::*;

fn configured_section(vapid_key: String) -> AnyView {
    view! {
        <section class="card">
            <h2>"Push Notifications"</h2>
            <p>"Receive a notification when your TripIt sync completes."</p>
            <div id="push-config" data-vapid-key=vapid_key></div>
            <button id="push-toggle" type="button" class="btn mt-sm" disabled>
                "Enable Push Notifications"
            </button>
            <p id="push-status" class="mt-sm">"Checking push notification status..."</p>
            <script defer src="/static/push.js"></script>
        </section>
    }
    .into_any()
}

fn unconfigured_section() -> AnyView {
    view! {
        <section class="card">
            <h2>"Push Notifications"</h2>
            <p>"Push notifications are not configured on this server"</p>
        </section>
    }
    .into_any()
}

#[component]
pub(super) fn PushSection(vapid_public_key: Option<String>) -> impl IntoView {
    match vapid_public_key {
        Some(vapid_key) => configured_section(vapid_key),
        None => unconfigured_section(),
    }
}
