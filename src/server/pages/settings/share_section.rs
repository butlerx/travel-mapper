use crate::db;
use leptos::prelude::*;

#[component]
pub(super) fn ShareSection(tokens: Vec<db::share_tokens::Row>, base_url: String) -> impl IntoView {
    view! {
        <section class="card">
            <h2>"Share Stats"</h2>
            <p>"Generate a public link to share your travel statistics. Anyone with the link can view your stats without logging in."</p>
            <form method="post" action="/auth/share-tokens" class="mt-sm">
                <label>"Label"</label>
                <div class="input-group">
                    <input type="text" name="label" placeholder="e.g. Year in Review" required />
                    <button type="submit" class="btn">"Generate Share Link"</button>
                </div>
            </form>

            {if tokens.is_empty() {
                view! { <p class="mt-sm text-muted">"No share tokens yet."</p> }.into_any()
            } else {
                view! {
                    <ul class="token-list mt-sm">
                        {tokens.into_iter().map(|t| {
                            let delete_url = format!("/auth/share-tokens/{}", t.id);
                            let full_url = format!("{base_url}/share/{}", t.token_hash);
                            view! {
                                <li class="token-list-item">
                                    <div class="token-info">
                                        <span class="token-label">{if t.label.is_empty() { "(no label)".to_owned() } else { t.label }}</span>
                                    </div>
                                    <div class="token-actions">
                                        <span class="new-token-value">
                                            <code data-copy-value=full_url style="display:none"></code>
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
