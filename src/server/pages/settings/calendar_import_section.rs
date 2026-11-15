use leptos::prelude::*;

#[component]
pub(super) fn CalendarImportSection() -> impl IntoView {
    view! {
        <section class="card">
            <h2>"Calendar Import"</h2>
            <p>"Import journeys from a calendar feed (.ics). Paste a public or webcal link \u{2014} for example a shared travel calendar or a TravelMapper feed URL."</p>
            <form method="POST" action="/import/ics">
                <div class="form-group">
                    <label for="ics-url">"Calendar URL"</label>
                    <input
                        type="text"
                        id="ics-url"
                        name="url"
                        placeholder="https://example.com/calendar.ics"
                        required
                    />
                </div>
                <button type="submit" class="btn btn-primary">"Import from Calendar"</button>
            </form>
        </section>
    }
}
