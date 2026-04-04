use leptos::prelude::*;

/// Shared filter form for stats and dashboard pages.
///
/// When `extended` is false (default), renders year and travel-type selects
/// (stats page). When true, adds search, origin, destination, dates, airline,
/// cabin class, and flight reason inputs (dashboard page).
#[component]
pub fn StatsFilters(
    available_years: Vec<String>,
    selected_year: Option<String>,
    selected_travel_type: Option<String>,
    #[prop(default = "/stats".to_owned())] action: String,
    #[prop(default = false)] extended: bool,
    #[prop(default = None)] selected_origin: Option<String>,
    #[prop(default = None)] selected_dest: Option<String>,
    #[prop(default = None)] selected_date_from: Option<String>,
    #[prop(default = None)] selected_date_to: Option<String>,
    #[prop(default = None)] selected_airline: Option<String>,
    #[prop(default = None)] selected_cabin_class: Option<String>,
    #[prop(default = None)] selected_flight_reason: Option<String>,
    #[prop(default = None)] selected_q: Option<String>,
) -> impl IntoView {
    view! {
        <form method="get" action=action.clone() class="stats-filters">
            {year_filter(available_years, selected_year)}
            {travel_type_filter(selected_travel_type)}
            {if extended {
                extended_filters(
                    &action,
                    ExtendedFilterValues {
                        q: selected_q,
                        origin: selected_origin,
                        dest: selected_dest,
                        date_from: selected_date_from,
                        date_to: selected_date_to,
                        airline: selected_airline,
                        cabin_class: selected_cabin_class,
                        flight_reason: selected_flight_reason,
                    },
                )
                .into_any()
            } else {
                ().into_any()
            }}
        </form>
    }
}

fn year_filter(available_years: Vec<String>, selected_year: Option<String>) -> impl IntoView {
    if available_years.is_empty() {
        ().into_any()
    } else {
        view! {
            <div class="stats-filter-group">
                <label for="year-filter">"Year:"</label>
                <select name="year" id="year-filter" data-auto-submit>
                    <option value="" selected=selected_year.is_none()>"All"</option>
                    {available_years.into_iter().rev().map(|y| {
                        let is_selected = selected_year.as_ref() == Some(&y);
                        let display = y.clone();
                        view! {
                            <option value={y} selected=is_selected>{display}</option>
                        }
                    }).collect::<Vec<_>>()}
                </select>
            </div>
        }
        .into_any()
    }
}

fn travel_type_filter(selected: Option<String>) -> impl IntoView {
    view! {
        <div class="stats-filter-group">
            <label for="travel-type-filter">"Type:"</label>
            <select name="travel_type" id="travel-type-filter" data-auto-submit>
                <option value="" selected=selected.is_none()>"All"</option>
                <option value="air" selected=selected.as_deref() == Some("air")>"Flights"</option>
                <option value="rail" selected=selected.as_deref() == Some("rail")>"Rail"</option>
                <option value="boat" selected=selected.as_deref() == Some("boat")>"Boat"</option>
                <option value="transport" selected=selected.as_deref() == Some("transport")>
                    "Transport"
                </option>
            </select>
        </div>
    }
}

struct ExtendedFilterValues {
    q: Option<String>,
    origin: Option<String>,
    dest: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    airline: Option<String>,
    cabin_class: Option<String>,
    flight_reason: Option<String>,
}

fn extended_filters(_action: &str, vals: ExtendedFilterValues) -> impl IntoView {
    view! {
        <div class="stats-filter-group">
            <label for="filter-q">"Search:"</label>
            <input
                type="text"
                name="q"
                id="filter-q"
                placeholder="Destinations, airlines\u{2026}"
                value=vals.q.unwrap_or_default()
                data-auto-submit
            />
        </div>
        <div class="stats-filter-group">
            <label for="filter-origin">"Origin:"</label>
            <input
                type="text"
                name="origin"
                id="filter-origin"
                placeholder="e.g. LHR"
                value=vals.origin.unwrap_or_default()
                data-auto-submit
            />
        </div>
        <div class="stats-filter-group">
            <label for="filter-dest">"Dest:"</label>
            <input
                type="text"
                name="dest"
                id="filter-dest"
                placeholder="e.g. JFK"
                value=vals.dest.unwrap_or_default()
                data-auto-submit
            />
        </div>
        <div class="stats-filter-group">
            <label for="filter-date-from">"From:"</label>
            <input
                type="date"
                name="date_from"
                id="filter-date-from"
                value=vals.date_from.unwrap_or_default()
                data-auto-submit
            />
        </div>
        <div class="stats-filter-group">
            <label for="filter-date-to">"To:"</label>
            <input
                type="date"
                name="date_to"
                id="filter-date-to"
                value=vals.date_to.unwrap_or_default()
                data-auto-submit
            />
        </div>
        <div class="stats-filter-group">
            <label for="filter-airline">"Airline:"</label>
            <input
                type="text"
                name="airline"
                id="filter-airline"
                placeholder="e.g. BA"
                value=vals.airline.unwrap_or_default()
                data-auto-submit
            />
        </div>
        {cabin_class_filter(vals.cabin_class)}
        {flight_reason_filter(vals.flight_reason)}
    }
}

fn cabin_class_filter(selected: Option<String>) -> impl IntoView {
    view! {
        <div class="stats-filter-group">
            <label for="filter-cabin">"Cabin:"</label>
            <select name="cabin_class" id="filter-cabin" data-auto-submit>
                <option value="" selected=selected.is_none()>"Any"</option>
                <option value="economy" selected=selected.as_deref() == Some("economy")>
                    "Economy"
                </option>
                <option
                    value="premium_economy"
                    selected=selected.as_deref() == Some("premium_economy")
                >
                    "Premium Economy"
                </option>
                <option value="business" selected=selected.as_deref() == Some("business")>
                    "Business"
                </option>
                <option value="first" selected=selected.as_deref() == Some("first")>"First"</option>
            </select>
        </div>
    }
}

fn flight_reason_filter(selected: Option<String>) -> impl IntoView {
    view! {
        <div class="stats-filter-group">
            <label for="filter-reason">"Reason:"</label>
            <select name="flight_reason" id="filter-reason" data-auto-submit>
                <option value="" selected=selected.is_none()>"Any"</option>
                <option value="personal" selected=selected.as_deref() == Some("personal")>
                    "Personal"
                </option>
                <option value="business" selected=selected.as_deref() == Some("business")>
                    "Business"
                </option>
            </select>
        </div>
    }
}
