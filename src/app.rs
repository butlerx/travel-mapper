// Leptos `#[component]` macro generates `#[must_use]` automatically,
// and component props must own their data (String, not &str).
#![allow(clippy::must_use_candidate, clippy::needless_pass_by_value)]

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

#[component]
pub fn LandingPage() -> impl IntoView {
    view! {
        <Shell title="Home".to_owned()>
            <main class="auth-page">
                <div class="card auth-card">
                    <div class="hero">
                        <h1>"Travel Export"</h1>
                        <p>"Sync your TripIt travel history to a local database and explore it via JSON, CSV, or HTML."</p>
                        <div class="hero-actions">
                            <a class="btn btn-primary" href="/register">"Get Started"</a>
                            <a class="btn btn-secondary" href="/login">"Log In"</a>
                        </div>
                    </div>
                </div>
            </main>
        </Shell>
    }
}

#[component]
pub fn AuthFormPage(
    title: &'static str,
    action: &'static str,
    submit_label: &'static str,
    footer_text: &'static str,
    footer_link_href: &'static str,
    footer_link_text: &'static str,
    autocomplete_password: &'static str,
    #[prop(optional_no_strip)] error: Option<String>,
) -> impl IntoView {
    view! {
        <Shell title=title.to_owned()>
            <main class="auth-page">
                <div class="card auth-card">
                    <h1>{title}</h1>
                    {error.map(|e| view! {
                        <div class="alert alert-error" role="alert">{e}</div>
                    })}
                    <form method="post" action=action>
                        <div class="form-group">
                            <label for="username">"Username"</label>
                            <input type="text" id="username" name="username" required autocomplete="username" placeholder=" " />
                        </div>
                        <div class="form-group">
                            <label for="password">"Password"</label>
                            <input type="password" id="password" name="password" required autocomplete=autocomplete_password placeholder=" " />
                        </div>
                        <button class="btn btn-primary btn-full" type="submit">{submit_label}</button>
                    </form>
                    <div class="form-footer">
                        <p>{footer_text} <a href=footer_link_href>{footer_link_text}</a></p>
                    </div>
                </div>
            </main>
        </Shell>
    }
}

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

#[component]
pub fn SettingsPage(
    has_tripit: bool,
    sync_status: Option<String>,
    last_sync_at: Option<String>,
    trips_fetched: Option<i64>,
    hops_fetched: Option<i64>,
    #[prop(optional_no_strip)] error: Option<String>,
    #[prop(optional_no_strip)] tripit_connected: Option<String>,
) -> impl IntoView {
    let has_sync = sync_status.is_some();
    view! {
        <Shell title="Settings".to_owned()>
            <NavBar current="settings" />
            <main class="container">
                {error.map(|e| view! {
                    <div class="alert alert-error" role="alert">{e}</div>
                })}
                {tripit_connected.filter(|v| v == "connected").map(|_| view! {
                    <div class="alert alert-success" role="status">"TripIt account connected successfully!"</div>
                })}

                <section class="card">
                    <h2>"TripIt Connection"</h2>
                    {if has_tripit {
                        view! { <span class="status-badge status-connected">"Connected"</span> }.into_any()
                    } else {
                        view! {
                            <span class="status-badge status-disconnected">"Not Connected"</span>
                            <div class="mt-md">
                                <a class="btn btn-primary" href="/auth/tripit/connect">"Connect TripIt"</a>
                            </div>
                        }.into_any()
                    }}
                </section>

                <section class="card">
                    <h2>"Sync Status"</h2>
                    {if has_sync {
                        view! {
                            <div class="stat-row">
                                <div class="stat-card">
                                    <div class="stat-label">"Status"</div>
                                    <div class="stat-value">{sync_status.unwrap_or_default()}</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-label">"Last Sync"</div>
                                    <div class="stat-value">{last_sync_at.unwrap_or_else(|| "never".to_owned())}</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-label">"Trips"</div>
                                    <div class="stat-value">{trips_fetched.unwrap_or(0)}</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-label">"Hops"</div>
                                    <div class="stat-value">{hops_fetched.unwrap_or(0)}</div>
                                </div>
                            </div>
                            {has_tripit.then(|| view! {
                                <form class="mt-md" method="post" action="/sync">
                                    <button class="btn btn-success" type="submit">"Sync Now"</button>
                                </form>
                            })}
                        }.into_any()
                    } else {
                        view! {
                            <div class="empty-state">
                                <div class="empty-state-icon">"~"</div>
                                <p>"No sync data yet."</p>
                            </div>
                        }.into_any()
                    }}
                </section>

                <section class="card">
                    <h2>"API Keys"</h2>
                    <p>"Use API keys for programmatic access to your travel data."</p>
                    <div class="mt-sm">
                        <code>"POST /auth/api-keys"</code>
                    </div>
                    <p class="mt-sm">"Create keys via the API with a session cookie or existing API key."</p>
                </section>
            </main>
        </Shell>
    }
}

#[component]
pub fn DashboardPage(
    hops_json: String,
    hop_count: usize,
    #[prop(optional_no_strip)] error: Option<String>,
) -> impl IntoView {
    let has_hops = hop_count > 0;
    let hops_script = format!("window.allHops={hops_json};");

    view! {
        <Shell title="Dashboard".to_owned()>
            <NavBar current="dashboard" />
            {error.map(|e| view! {
                <div class="alert alert-error" role="alert">{e}</div>
            })}

            {if has_hops {
                view! {
                    <div id="map"></div>
                    <div class="map-controls">
                        <div class="map-filters">
                            <label for="filter-type">{"\u{1F3F7}\u{FE0F} Type"}</label>
                            <select id="filter-type">
                                <option value="all">"All Types"</option>
                                <option value="air">{"\u{2708}\u{FE0F} Air"}</option>
                                <option value="rail">{"\u{1F686} Rail"}</option>
                                <option value="cruise">{"\u{1F6A2} Cruise"}</option>
                                <option value="transport">{"\u{1F697} Transport"}</option>
                            </select>
                            <label for="filter-year">{"\u{1F4C5} Year"}</label>
                            <select id="filter-year">
                                <option value="all">"All Years"</option>
                            </select>
                        </div>
                        <div class="map-legend">
                            <h3>{"\u{1F5FA}\u{FE0F} Routes"}</h3>
                            <div class="legend-item">
                                <div class="legend-swatch legend-air"></div>
                                <span>{"\u{2708}\u{FE0F} Air"}</span>
                            </div>
                            <div class="legend-item">
                                <div class="legend-swatch legend-rail"></div>
                                <span>{"\u{1F686} Rail"}</span>
                            </div>
                            <div class="legend-item">
                                <div class="legend-swatch legend-cruise"></div>
                                <span>{"\u{1F6A2} Cruise"}</span>
                            </div>
                            <div class="legend-item">
                                <div class="legend-swatch legend-transport"></div>
                                <span>{"\u{1F697} Transport"}</span>
                            </div>
                            <div class="legend-count" id="hop-count">{hop_count}" hops"</div>
                        </div>
                    </div>
                    <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
                        integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY="
                        crossorigin="" />
                    <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"
                        integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo="
                        crossorigin=""></script>
                    <script inner_html=hops_script></script>
                    <script src="/static/map.js"></script>
                }.into_any()
            } else {
                view! {
                    <main class="container-wide">
                        <section class="card">
                            <div class="empty-state">
                                <div class="empty-state-icon">{"\u{1F30D}"}</div>
                                <p>"No hops yet. Connect TripIt in " <a href="/settings">"Settings"</a> " and sync to see your travel data."</p>
                            </div>
                        </section>
                    </main>
                }.into_any()
            }}
        </Shell>
    }
}

pub fn shell(_options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <link rel="stylesheet" href="/static/style.css" />
            </head>
            <body>
                <main />
            </body>
        </html>
    }
}
