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

    view! {
        <section class="card">
            <h2>"Email"</h2>
            <p>
                "Status: "
                <span class=status_class>{status_text}</span>
                {(!email.is_empty()).then(|| {
                    let e = email.clone();
                    view! {
                        <span class="ml-sm">{format!("({e})")}</span>
                    }
                })}
            </p>

            {(!email_verified && !email.is_empty()).then(|| view! {
                <form method="post" action="/auth/resend-verification" class="mt-sm">
                    <button type="submit" class="btn btn-sm">"Resend Verification Email"</button>
                </form>
            })}

            <h3 class="mt-sm">"Update Email"</h3>
            <form method="post" action="/auth/email" class="mt-sm">
                <label>
                    "Email address"
                    <input
                        type="email"
                        name="email"
                        placeholder="you@example.com"
                        value=email
                    />
                </label>
                <button type="submit" class="btn mt-sm">"Save Email"</button>
            </form>
        </section>
    }
}
