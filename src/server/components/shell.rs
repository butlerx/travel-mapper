use crate::server::{APP_NAME, THEME_COLOR};
use leptos::prelude::*;

/// HTML document shell wrapping page content with head metadata and scripts.
#[component]
pub fn Shell(
    title: String,
    #[prop(optional)] body_class: Option<&'static str>,
    #[prop(optional)] og_meta: Option<String>,
    children: Children,
) -> impl IntoView {
    let class = body_class.unwrap_or_default();

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="theme-color" content=THEME_COLOR />
                <meta name="apple-mobile-web-app-capable" content="yes" />
                <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
                <title>{format!("{title} — {APP_NAME}")}</title>
                <link rel="icon" type="image/svg+xml" href="/static/icons/logo.svg" />
                <link rel="apple-touch-icon" href="/static/logo.svg" />
                <link rel="manifest" href="/manifest.json" />
                <link rel="stylesheet" href="/static/style.css" />
                {og_meta.map(|m| view! { <div class="hidden" inner_html=m /> })}
                <script defer src="/static/nav.js"></script>
                <script defer src="/static/auto-submit.js"></script>
                <script defer src="/static/sw-register.js"></script>
            </head>
            <body class=class>
                {children()}
            </body>
            <footer>
                <p>"Self-hosted \u{00B7}"</p><a href={env!("CARGO_PKG_HOMEPAGE")}>"Open source"</a><p>"\u{00B7} Your data stays yours"</p>
            </footer>
        </html>
    }
}
