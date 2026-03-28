use super::shell::Shell;
use leptos::prelude::*;

/// Shared layout for login and registration form pages.
#[component]
pub fn AuthFormPage(
    title: &'static str,
    action: &'static str,
    submit_label: &'static str,
    footer_text: &'static str,
    footer_link_href: &'static str,
    footer_link_text: &'static str,
    autocomplete_password: &'static str,
    #[prop(optional_no_strip)] error: Option<String>,
    #[prop(default = true)] show_footer: bool,
) -> impl IntoView {
    view! {
        <Shell title=title.to_owned()>
            <main class="auth-page">
                <div class="card auth-card">
                    <h1>{title}</h1>
                    {error.map(|e| view! {
                        <div class="alert alert-error" role="alert">{e}</div>
                    })}
                    <form method="post" action=action>
                        <div class="form-group">
                            <label for="username">"Username"</label>
                            <input type="text" id="username" name="username" required autocomplete="username" placeholder=" " />
                        </div>
                        <div class="form-group">
                            <label for="password">"Password"</label>
                            <input type="password" id="password" name="password" required autocomplete=autocomplete_password placeholder=" " />
                        </div>
                        <button class="btn btn-primary btn-full" type="submit">{submit_label}</button>
                    </form>
                    {show_footer.then(|| view! {
                        <div class="form-footer">
                            <p>{footer_text} <a href=footer_link_href>{footer_link_text}</a></p>
                        </div>
                    })}
                </div>
            </main>
        </Shell>
    }
}
