use leptos::prelude::*;

#[component]
pub fn Shell(
    title: String,
    #[prop(optional)] body_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let class = body_class.unwrap_or_default();
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <title>{format!("{title} — Travel Export")}</title>
                <link rel="icon" type="image/svg+xml" href="/static/logo.svg" />
                <link rel="stylesheet" href="/static/style.css" />
            </head>
            <body class=class>
                {children()}
            </body>
        </html>
    }
}
