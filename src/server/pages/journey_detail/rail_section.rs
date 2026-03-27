use super::detail_row_view;
use crate::db;
use leptos::prelude::*;

pub(super) struct RailEnrichmentView {
    pub dep_platform: String,
    pub arr_platform: String,
    pub provider: String,
}

fn platform_row(label: &str, platform: &str) -> AnyView {
    if platform.is_empty() {
        ().into_any()
    } else {
        view! {
            <div class="journey-detail-label">{label.to_owned()}</div>
            <div class="journey-detail-value">
                <span class="platform-badge">{format!("Platform {platform}")}</span>
            </div>
        }
        .into_any()
    }
}

fn provider_row(provider: &str) -> AnyView {
    if provider.is_empty() {
        ().into_any()
    } else {
        let label = match provider {
            "db_ris" => "via Deutsche Bahn",
            "darwin" => "via National Rail",
            "transitland" => "via Transitland GTFS",
            "amtrak" => "via Amtrak",
            other => other,
        };
        view! {
            <div class="journey-detail-label">"Source"</div>
            <div class="journey-detail-value journey-detail-muted">{label.to_owned()}</div>
        }
        .into_any()
    }
}

#[component]
pub(super) fn RailSection(
    detail: db::hops::RailDetail,
    #[prop(optional_no_strip)] enrichment: Option<RailEnrichmentView>,
) -> impl IntoView {
    let dep_platform = enrichment.as_ref().map(|e| e.dep_platform.clone());
    let arr_platform = enrichment.as_ref().map(|e| e.arr_platform.clone());
    let provider = enrichment.as_ref().map(|e| e.provider.clone());

    view! {
        <section class="journey-detail-section">
            <h3>"Rail Details"</h3>
            <div class="journey-detail-grid">
                {detail_row_view("Carrier", &detail.carrier)}
                {detail_row_view("Train", &detail.train_number)}
                {dep_platform.as_deref().map(|p| platform_row("Departure Platform", p))}
                {arr_platform.as_deref().map(|p| platform_row("Arrival Platform", p))}
                {detail_row_view("Class", &detail.service_class)}
                {detail_row_view("Coach", &detail.coach_number)}
                {detail_row_view("Seats", &detail.seats)}
                {detail_row_view("Confirmation", &detail.confirmation_num)}
                {detail_row_view("Booking Site", &detail.booking_site)}
                {provider.as_deref().map(provider_row)}
            </div>
        </section>
        {if detail.notes.is_empty() { ().into_any() } else {
            let notes = detail.notes.clone();
            view! {
                <section class="journey-detail-section">
                    <h3>"Notes"</h3>
                    <p class="journey-detail-notes">{notes}</p>
                </section>
            }.into_any()
        }}
    }
}
