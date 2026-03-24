use leptos::prelude::*;

#[component]
pub fn NavBar(#[prop(into)] current: String) -> impl IntoView {
    let aria = |name: &str| if current == name { Some("page") } else { None };

    view! {
        <nav class="nav" aria-label="Main">
            <a class="nav-brand" href="/dashboard">
                <img class="nav-logo" src="/static/logo.svg" alt="" width="32" height="32" />
                "Travel Export"
            </a>
            <div class="spacer"></div>
            <a class="nav-link" href="/dashboard" aria-current=aria("dashboard")>"Dashboard"</a>
            <a class="nav-link" href="/hops" aria-current=aria("hops")>"Hops"</a>
            <a class="nav-link" href="/trips" aria-current=aria("trips")>"Trips"</a>
            <a class="nav-link" href="/hops/new" aria-current=aria("add-hop")>"Add Hop"</a>
            <a class="nav-link" href="/stats" aria-current=aria("stats")>"Stats"</a>
            <a class="nav-link" href="/settings" aria-current=aria("settings")>"Settings"</a>
            <form method="post" action="/auth/logout" style="margin:0">
                <button class="btn btn-danger" type="submit">"Log Out"</button>
            </form>
        </nav>
    }
}
