use leptos::prelude::*;

#[component]
pub(super) fn ShareSection() -> impl IntoView {
    view! {
        <section class="card">
            <h2>"Share Stats"</h2>
            <p>"Generate a public link to share your travel statistics. Anyone with the link can view your stats without logging in."</p>

            <h3 class="mt-sm">"Create a Share Token"</h3>
            <form method="post" action="/auth/share-tokens" class="mt-sm">
                <label>
                    "Label (optional)"
                    <input type="text" name="label" placeholder="e.g. Year in Review" />
                </label>
                <button type="submit" class="btn mt-sm">"Generate Share Link"</button>
            </form>

            <h3 class="mt-sm">"How to Use"</h3>
            <ol class="mt-sm">
                <li>"Create a share token above (or via "<code>"POST /auth/share-tokens"</code>")."</li>
                <li>"Copy the token from the response."</li>
                <li>"Share this URL: "<code>"/share/TOKEN"</code></li>
                <li>"Add "<code>"?year=2025"</code>" for a specific year."</li>
            </ol>
            <p class="mt-sm">"To revoke a share link, delete its token via "<code>"DELETE /auth/share-tokens/:id"</code>"."</p>
        </section>
    }
}
