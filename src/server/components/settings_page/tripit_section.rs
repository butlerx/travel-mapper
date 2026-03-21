use leptos::prelude::*;

#[component]
pub fn TripitSection(has_tripit: bool) -> impl IntoView {
    view! {
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
    }
}
