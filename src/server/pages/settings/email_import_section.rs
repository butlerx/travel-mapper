use crate::db;
use leptos::prelude::*;

#[component]
pub(super) fn EmailImportSection(
    tokens: Vec<db::inbound_email_tokens::Row>,
    #[prop(optional_no_strip)] new_token: Option<String>,
    base_url: String,
) -> impl IntoView {
    let domain = base_url
        .strip_prefix("https://")
        .or_else(|| base_url.strip_prefix("http://"))
        .unwrap_or("your-domain.com");

    view! {
        <section class="card">
            <h2>"Email Import"</h2>
            <p>"Forward booking confirmation emails to a unique address to auto-import flights and journeys."</p>

            {new_token.map(|token| {
                let address = format!("import-{token}@{domain}");
                view! {
                    <div class="alert alert-success" role="status">
                        <strong>"Your forwarding address:"</strong>
                        <code>{address.clone()}</code>
                        <p class="text-muted">"Forward booking confirmations from airlines and travel providers to this address."</p>
                    </div>
                }
            })}

            <form method="post" action="/email/tokens" class="mt-sm">
                <label>"Label"</label>
                <div class="input-group">
                    <input type="text" name="label" placeholder="e.g. Gmail forwarding" />
                    <button type="submit" class="btn btn-primary">"Generate Address"</button>
                </div>
            </form>

            {if tokens.is_empty() {
                view! { <p class="mt-sm text-muted">"No forwarding addresses configured yet."</p> }.into_any()
            } else {
                view! {
                    <ul class="token-list mt-sm">
                        {tokens.into_iter().map(|t| {
                            let address = format!("import-{}@{domain}", t.token_hash);
                            view! {
                                <li class="token-list-item">
                                    <div class="token-info">
                                        <span class="token-label">{if t.label.is_empty() { "(no label)".to_owned() } else { t.label }}</span>
                                        <span class="text-muted">{address}</span>
                                    </div>
                                    <div class="token-actions">
                                        <form method="post" action="/email/tokens/delete">
                                            <input type="hidden" name="id" value=t.id.to_string() />
                                            <button type="submit" class="btn btn-sm btn-danger">"Revoke"</button>
                                        </form>
                                    </div>
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                }.into_any()
            }}

            <details class="mt-sm">
                <summary class="text-muted">"How it works"</summary>
                <ol class="text-muted">
                    <li>"Generate a forwarding address above."</li>
                    <li>"Set up an email forwarding rule (e.g. in Gmail: filter booking emails → forward to your address)."</li>
                    <li>"Configure your email provider (SendGrid, Mailgun, or any webhook-capable service) to POST incoming emails to "<code>{format!("{base_url}/email/webhook")}</code>"."</li>
                    <li>"Supported formats: schema.org JSON-LD, ICS attachments, or text with IATA route codes."</li>
                </ol>
            </details>
        </section>
    }
}
