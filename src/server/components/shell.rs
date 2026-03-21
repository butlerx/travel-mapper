use leptos::prelude::*;

#[component]
pub fn Shell(title: String, children: Children) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <title>{format!("{title} — Travel Export")}</title>
                <link rel="stylesheet" href="/static/style.css" />
            </head>
            <body>
                {children()}
            </body>
        </html>
    }
}
