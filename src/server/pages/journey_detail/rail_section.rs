use super::detail_row_view;
use crate::db;
use leptos::prelude::*;

#[component]
pub(super) fn RailSection(detail: db::hops::RailDetail) -> impl IntoView {
    view! {
        <section class="journey-detail-section">
            <h3>"Rail Details"</h3>
            <div class="journey-detail-grid">
                {detail_row_view("Carrier", &detail.carrier)}
                {detail_row_view("Train", &detail.train_number)}
                {detail_row_view("Class", &detail.service_class)}
                {detail_row_view("Coach", &detail.coach_number)}
                {detail_row_view("Seats", &detail.seats)}
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
