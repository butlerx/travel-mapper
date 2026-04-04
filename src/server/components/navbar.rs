use crate::server::APP_NAME;
use leptos::prelude::*;

/// Top navigation bar with responsive hamburger menu.
#[component]
pub fn NavBar(#[prop(into)] current: String) -> impl IntoView {
    let aria = |name: &str| if current == name { Some("page") } else { None };

    view! {
        <nav class="nav" aria-label="Main">
            <a class="nav-brand" href="/dashboard">
                <img class="nav-logo" src="/static/icons/logo.svg" alt="" width="32" height="32" />
                {APP_NAME}
            </a>
            <div class="spacer"></div>
            <button
                class="nav-toggle"
                type="button"
                aria-expanded="false"
                aria-controls="nav-menu"
                aria-label="Toggle navigation"
            >
                <span class="nav-toggle-icon" aria-hidden="true"></span>
            </button>
            <div id="nav-menu" class="nav-menu">
                <a class="nav-link" href="/dashboard" aria-current=aria("dashboard")>"Dashboard"</a>
                <a class="nav-link" href="/journeys" aria-current=aria("journeys")>"Journeys"</a>
                <a class="nav-link" href="/trips" aria-current=aria("trips")>"Trips"</a>
                <a class="nav-link" href="/journeys/new" aria-current=aria("add-journey")>"Add Journey"</a>
                <a class="nav-link" href="/stats" aria-current=aria("stats")>"Stats"</a>
                <a class="nav-link" href="/settings" aria-current=aria("settings")>"Settings"</a>
                <form method="post" action="/auth/logout">
                    <button class="btn btn-danger" type="submit">"Log Out"</button>
                </form>
            </div>
        </nav>
    }
}
