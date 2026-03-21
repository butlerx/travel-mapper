use leptos::prelude::*;

#[component]
pub fn NavBar(#[prop(into)] current: String) -> impl IntoView {
    let aria = |name: &str| if current == name { Some("page") } else { None };

    view! {
        <nav class="nav" aria-label="Main">
            <a class="nav-brand" href="/dashboard">"Travel Export"</a>
            <div class="spacer"></div>
            <a class="nav-link" href="/dashboard" aria-current=aria("dashboard")>"Dashboard"</a>
            <a class="nav-link" href="/hops" aria-current=aria("hops")>"Hops"</a>
            <a class="nav-link" href="/settings" aria-current=aria("settings")>"Settings"</a>
            <form method="post" action="/auth/logout" style="margin:0">
                <button class="btn btn-danger" type="submit">"Log Out"</button>
            </form>
        </nav>
    }
}
