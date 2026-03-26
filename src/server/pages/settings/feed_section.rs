use leptos::prelude::*;

#[component]
pub(super) fn FeedSection() -> impl IntoView {
    view! {
        <section class="card">
            <h2>"Calendar Feed"</h2>
            <p>"Subscribe to your travel schedule in any calendar app (Google Calendar, Apple Calendar, Outlook, etc.)."</p>

            <h3 class="mt-sm">"Create a Feed Token"</h3>
            <form method="post" action="/auth/feed-tokens" class="mt-sm">
                <label>
                    "Label (optional)"
                    <input type="text" name="label" placeholder="e.g. My iPhone" />
                </label>
                <button type="submit" class="btn mt-sm">"Generate Feed URL"</button>
            </form>

            <h3 class="mt-sm">"How to Subscribe"</h3>
            <ol class="mt-sm">
                <li>"Create a feed token above (or via "<code>"POST /auth/feed-tokens"</code>")."</li>
                <li>"Copy the token from the response."</li>
                <li>"Add this URL to your calendar app: "<code>"/feed/TOKEN.ics"</code></li>
            </ol>
            <p class="mt-sm">"To revoke a feed, delete its token via "<code>"DELETE /auth/feed-tokens/:id"</code>"."</p>
        </section>
    }
}
