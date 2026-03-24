use super::detail_row_view;
use crate::db;
use leptos::prelude::*;

#[component]
pub(super) fn TransportSection(detail: db::hops::TransportDetail) -> impl IntoView {
    view! {
        <section class="journey-detail-section">
            <h3>"Transport Details"</h3>
            <div class="journey-detail-grid">
                {detail_row_view("Carrier", &detail.carrier_name)}
                {detail_row_view("Vehicle", &detail.vehicle_description)}
                {detail_row_view("Confirmation", &detail.confirmation_num)}
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
