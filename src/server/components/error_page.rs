use super::shell::Shell;
use leptos::prelude::*;

/// Styled error page displaying a status code and message.
#[component]
pub fn ErrorPage(
    code: &'static str,
    title: &'static str,
    message: &'static str,
    link_href: &'static str,
    link_text: &'static str,
) -> impl IntoView {
    view! {
        <Shell title=title.to_owned()>
            <main class="error-page">
                <div class="error-code">{code}</div>
                <h1>{title}</h1>
                <p>{message}</p>
                <a class="btn btn-primary" href=link_href>{link_text}</a>
            </main>
        </Shell>
    }
}
