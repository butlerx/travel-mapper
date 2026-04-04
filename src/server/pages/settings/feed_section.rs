use crate::db;
use leptos::prelude::*;

#[component]
pub(super) fn FeedSection(tokens: Vec<db::feed_tokens::Row>, base_url: String) -> impl IntoView {
    view! {
        <section class="card">
            <h2>"Calendar Feed"</h2>
            <p>"Subscribe to your travel schedule in any calendar app."</p>
            <form method="post" action="/auth/feed-tokens" class="mt-sm">
                <label>"Label"</label>
                <div class="input-group">
                    <input type="text" name="label" placeholder="e.g. My iPhone" required />
                    <button type="submit" class="btn">"Generate Feed URL"</button>
                </div>
            </form>

            {if tokens.is_empty() {
                view! { <p class="mt-sm text-muted">"No feed tokens yet."</p> }.into_any()
            } else {
                view! {
                    <ul class="token-list mt-sm">
                        {tokens.into_iter().map(|t| {
                            let delete_url = format!("/auth/feed-tokens/{}", t.id);
                            let full_url = format!("{base_url}/feed/{}.ics", t.token_hash);
                            view! {
                                <li class="token-list-item">
                                    <div class="token-info">
                                        <span class="token-label">{if t.label.is_empty() { "(no label)".to_owned() } else { t.label }}</span>
                                    </div>
                                    <div class="token-actions">
                                        <span class="new-token-value">
                                            <code class="hidden" data-copy-value=full_url></code>
                                            <button type="button" class="btn btn-sm btn-primary" data-copy-trigger>"Copy URL"</button>
                                        </span>
                                        <form method="post" action=delete_url>
                                            <button type="submit" class="btn btn-sm btn-danger">"Revoke"</button>
                                        </form>
                                    </div>
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                }.into_any()
            }}
        </section>
    }
}
