use leptos::prelude::*;

/// An item label paired with its occurrence count.
#[derive(Default, Clone)]
pub struct CountedItem {
    pub name: String,
    pub count: usize,
}

#[component]
pub fn TopList(title: &'static str, items: Vec<CountedItem>) -> impl IntoView {
    let max_count = items.first().map_or(1, |i| i.count.max(1));

    view! {
        <section class="stats-section">
            <h3 class="stats-section-title">{title}</h3>
            {if items.is_empty() {
                view! { <p class="stats-empty">"No data"</p> }.into_any()
            } else {
                view! {
                    <ul class="stats-top-list">
                        {items.into_iter().map(|item| {
                            let pct = item.count * 100 / max_count;
                            let width = format!("--pct: {pct}%");
                            view! {
                                <li class="stats-top-item">
                                    <div class="stats-top-bar" style=width></div>
                                    <span class="stats-top-name">{item.name}</span>
                                    <span class="stats-top-count">{item.count}</span>
                                </li>
                            }
                        }).collect::<Vec<_>>()}
                    </ul>
                }.into_any()
            }}
        </section>
    }
}

pub fn optional_top_list(title: &'static str, items: Vec<CountedItem>) -> AnyView {
    if items.is_empty() {
        ().into_any()
    } else {
        view! { <TopList title=title items=items /> }.into_any()
    }
}

pub fn spending_section_view(spending_summary: Vec<String>) -> AnyView {
    if spending_summary.is_empty() {
        ().into_any()
    } else {
        view! {
            <section class="stats-section">
                <h3 class="stats-section-title">"Spending"</h3>
                <ul class="stats-top-list">
                    {spending_summary.into_iter().map(|s| view! {
                        <li class="stats-top-item">
                            <span class="stats-top-name">{s}</span>
                        </li>
                    }).collect::<Vec<_>>()}
                </ul>
            </section>
        }
        .into_any()
    }
}

pub fn miles_section_view(miles_summary: Vec<String>) -> AnyView {
    if miles_summary.is_empty() {
        ().into_any()
    } else {
        view! {
            <section class="stats-section">
                <h3 class="stats-section-title">"Miles by Program"</h3>
                <ul class="stats-top-list">
                    {miles_summary.into_iter().map(|s| view! {
                        <li class="stats-top-item">
                            <span class="stats-top-name">{s}</span>
                        </li>
                    }).collect::<Vec<_>>()}
                </ul>
            </section>
        }
        .into_any()
    }
}
