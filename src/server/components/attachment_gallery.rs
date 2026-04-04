use crate::db;
use leptos::prelude::*;

#[component]
pub fn AttachmentGallery(journey_id: i64, attachments: Vec<db::attachments::Row>) -> impl IntoView {
    let upload_action = format!("/journeys/{journey_id}/attachments");

    view! {
        <section class="attachment-gallery">
            <h2>"Photos"</h2>
            {if attachments.is_empty() {
                view! { <p class="attachment-empty">"No photos attached yet."</p> }.into_any()
            } else {
                let items: Vec<AnyView> = attachments
                    .iter()
                    .map(|att| {
                        let src = format!("/journeys/{journey_id}/attachments/{}", att.id);
                        let delete_action = format!("/journeys/{journey_id}/attachments/{}", att.id);
                        let alt = att.filename.clone();
                        let filename = att.filename.clone();
                        let href = src.clone();
                        let img_src = src;
                        view! {
                            <div class="attachment-card">
                                <a href=href target="_blank">
                                    <img src=img_src alt=alt class="attachment-thumb" loading="lazy" />
                                </a>
                                <div class="attachment-info">
                                    <span class="attachment-name">{filename}</span>
                                    <form method="post" action=delete_action class="attachment-delete-form">
                                        <input type="hidden" name="_method" value="DELETE" />
                                        <button type="submit" class="btn btn-danger btn-sm">"Remove"</button>
                                    </form>
                                </div>
                            </div>
                        }
                        .into_any()
                    })
                    .collect();
                view! { <div class="attachment-grid">{items}</div> }.into_any()
            }}
            <form
                method="post"
                action=upload_action
                enctype="multipart/form-data"
                class="attachment-upload-form"
            >
                <label class="btn btn-secondary">
                    "Add Photos"
                    <input type="file" name="file" accept="image/*" multiple=true hidden=true />
                </label>
            </form>
        </section>
    }
}
