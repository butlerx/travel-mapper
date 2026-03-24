use super::detail_row_view;
use crate::db;
use leptos::prelude::*;

#[component]
pub(super) fn BoatSection(detail: db::hops::BoatDetail) -> impl IntoView {
    view! {
        <section class="journey-detail-section">
            <h3>"Boat Details"</h3>
            <div class="journey-detail-grid">
                {detail_row_view("Ship", &detail.ship_name)}
                {detail_row_view("Cabin Type", &detail.cabin_type)}
                {detail_row_view("Cabin Number", &detail.cabin_number)}
                {detail_row_view("Confirmation", &detail.confirmation_num)}
                {detail_row_view("Booking Site", &detail.booking_site)}
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
