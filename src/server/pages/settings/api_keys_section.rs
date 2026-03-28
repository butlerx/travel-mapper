use crate::db;
use leptos::prelude::*;

#[component]
pub(super) fn ApiKeysSection(
    keys: Vec<db::api_keys::Row>,
    #[prop(optional_no_strip)] new_key: Option<String>,
) -> impl IntoView {
    view! {
        <section class="card">
            <h2>"API Keys"</h2>
            <p>"Use API keys for programmatic access to your travel data."</p>

            <form method="post" action="/auth/api-keys" class="mt-sm">
                <label>"Label"</label>
                <div class="input-group">
                    <input type="text" name="label" placeholder="e.g. My Script" required />
                    <button type="submit" class="btn">"Create API Key"</button>
                </div>
            </form>

            {new_key.map(|key| {
                let key_attr = key.clone();
                view! {
                <div class="new-token-banner mt-sm">
                    <p class="new-token-heading">"API key created! Copy it now \u{2014} it won\u{2019}t be shown again."</p>
                    <div class="new-token-value">
                        <code data-copy-value=key_attr>{key}</code>
                        <button type="button" class="btn btn-sm copy-btn" data-copy-trigger>"Copy"</button>
                    </div>
                </div>
            }})}


            {if keys.is_empty() {
                view! { <p class="mt-sm text-muted">"No API keys yet."</p> }.into_any()
            } else {
                view! {
                    <ul class="token-list mt-sm">
                        {keys.into_iter().map(|k| {
                            let delete_url = format!("/auth/api-keys/{}", k.id);
                            view! {
                                <li class="token-list-item">
                                    <div class="token-info">
                                        <span class="token-label">{if k.label.is_empty() { "(no label)".to_owned() } else { k.label }}</span>
                                        <span class="token-date">{k.created_at}</span>
                                    </div>
                                    <form method="post" action=delete_url>
                                        <button type="submit" class="btn btn-sm btn-danger">"Revoke"</button>
                                    </form>
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                }.into_any()
            }}
        </section>
    }
}
