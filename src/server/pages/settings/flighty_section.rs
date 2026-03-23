use leptos::prelude::*;

#[component]
pub(super) fn FlightySection() -> impl IntoView {
    view! {
        <section class="card">
            <h2>"Flighty Import"</h2>
            <p>"Upload a Flighty CSV export to import your flight history."</p>
            <form method="POST" action="/import/flighty" enctype="multipart/form-data">
                <div class="form-group">
                    <label for="flighty-file">"Flighty CSV file"</label>
                    <input
                        type="file"
                        id="flighty-file"
                        name="file"
                        accept=".csv,text/csv"
                        required
                        class="file-input"
                    />
                </div>
                <button type="submit" class="btn btn-primary">"Import Flights"</button>
            </form>
        </section>
    }
}
