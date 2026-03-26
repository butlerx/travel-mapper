use leptos::prelude::*;

#[component]
pub(super) fn ProfileSection(first_name: String, last_name: String) -> impl IntoView {
    view! {
        <section class="card">
            <h2>"Profile"</h2>
            <form method="post" action="/auth/profile">
                <label>
                    "First Name"
                    <input
                        type="text"
                        name="first_name"
                        placeholder="First name"
                        value=first_name
                    />
                </label>
                <label>
                    "Last Name"
                    <input
                        type="text"
                        name="last_name"
                        placeholder="Last name"
                        value=last_name
                    />
                </label>
                <button type="submit" class="btn mt-sm">"Save Profile"</button>
            </form>
        </section>
    }
}
