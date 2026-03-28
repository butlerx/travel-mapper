use leptos::prelude::*;

#[component]
pub(super) fn EmailSection(email: String, email_verified: bool) -> impl IntoView {
    let status_text = match (email.is_empty(), email_verified) {
        (true, _) => "No email set",
        (false, true) => "Verified",
        (false, false) => "Unverified",
    };

    let status_class = match (email.is_empty(), email_verified) {
        (true, _) => "badge badge-muted",
        (false, true) => "badge badge-success",
        (false, false) => "badge badge-warning",
    };

    let placeholder = if email.is_empty() {
        "you@example.com"
    } else {
        &email
    };

    view! {
        <section class="card">
            <h2>"Email"</h2>
            <div class="status-row">
                <p>"Status: "<span class=status_class>{status_text}</span></p>
                {(!email_verified && !email.is_empty()).then(|| view! {
                    <form method="post" action="/auth/resend-verification">
                        <button type="submit" class="btn btn-sm btn-warning">"Resend Verification"</button>
                    </form>
                })}
            </div>

            <h3 class="mt-sm">"Update Email"</h3>
            <form method="post" action="/auth/email" class="mt-sm">
                <label>"Email address"</label>
                <div class="input-group">
                    <input
                        type="email"
                        name="email"
                        placeholder={placeholder.to_string()}
                        value=email
                    />
                    <button type="submit" class="btn">"Save"</button>
                </div>
            </form>
        </section>
    }
}
