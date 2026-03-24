use leptos::prelude::*;

#[component]
pub(super) fn CsvImportSection() -> impl IntoView {
    view! {
        <section class="card">
            <h2>"CSV Import"</h2>
            <p>"Upload a flight history export from Flighty, myFlightradar24, OpenFlights, or App in the Air."</p>
            <form method="POST" action="/import/csv" enctype="multipart/form-data">
                <div class="form-group">
                    <label for="csv-format">"Format"</label>
                    <select id="csv-format" name="format">
                        <option value="" selected>"Auto-detect"</option>
                        <option value="flighty">"Flighty"</option>
                        <option value="myflightradar24">"myFlightradar24"</option>
                        <option value="openflights">"OpenFlights"</option>
                        <option value="appintheair">"App in the Air"</option>
                    </select>
                </div>
                <div class="form-group">
                    <label for="csv-file">"CSV / data file"</label>
                    <input
                        type="file"
                        id="csv-file"
                        name="file"
                        accept=".csv,.txt,text/csv,text/plain"
                        required
                        class="file-input"
                    />
                </div>
                <button type="submit" class="btn btn-primary">"Import Flights"</button>
            </form>
        </section>
    }
}
