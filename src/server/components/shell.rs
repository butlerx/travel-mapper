use crate::server::{APP_NAME, THEME_COLOR};
use leptos::prelude::*;

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
                <link rel="icon" type="image/svg+xml" href="/static/logo.svg" />
                <link rel="apple-touch-icon" href="/static/logo.svg" />
                <link rel="manifest" href="/manifest.json" />
                <link rel="stylesheet" href="/static/style.css" />
                {og_meta.map(|m| view! { <div inner_html=m style="display:none" /> })}
                <script defer src="/static/nav.js"></script>
            </head>
            <body class=class>
                {children()}
                <script>
                    "if ('serviceWorker' in navigator) { navigator.serviceWorker.register('/sw.js'); }"
                </script>
            </body>
        </html>
    }
}
