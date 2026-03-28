use super::ErrorResponse;
use crate::{
    db,
    server::{
        AppState,
        error::AppError,
        extractors::AuthUser,
        routes::{ResponseFormat, negotiate_format},
    },
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use schemars::JsonSchema;
use serde::Serialize;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10 MB

const ALLOWED_CONTENT_TYPES: &[&str] = &["image/jpeg", "image/png", "image/gif", "image/webp"];

fn extension_for_content_type(ct: &str) -> &'static str {
    match ct {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "bin",
    }
}

/// JSON response for a file attachment.
#[derive(Debug, Serialize, JsonSchema)]
pub struct AttachmentResponse {
    pub id: i64,
    pub hop_id: i64,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub created_at: String,
}

impl From<db::attachments::Row> for AttachmentResponse {
    fn from(row: db::attachments::Row) -> Self {
        Self {
            id: row.id,
            hop_id: row.hop_id,
            filename: row.filename,
            content_type: row.content_type,
            size_bytes: row.size_bytes,
            created_at: row.created_at,
        }
    }
}

fn require_storage_path(state: &AppState) -> Result<&PathBuf, AppError> {
    state.storage_path.as_ref().ok_or(AppError::StorageDisabled)
}

async fn upload_inner(
    state: AppState,
    auth: AuthUser,
    journey_id: i64,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    let base_path = require_storage_path(&state)?;

    db::hops::GetById {
        id: journey_id,
        user_id: auth.user_id,
    }
    .execute(&state.db)
    .await?
    .ok_or_else(|| AppError::MissingField("journey not found"))?;

    let mut created: Vec<AttachmentResponse> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::MissingField("failed to read multipart field"))?
    {
        let original_filename = field.file_name().unwrap_or("upload").to_owned();

        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_owned();

        if !ALLOWED_CONTENT_TYPES.contains(&content_type.as_str()) {
            return Err(AppError::UnsupportedMediaType(content_type));
        }

        let data = field
            .bytes()
            .await
            .map_err(|_| AppError::MissingField("failed to read field bytes"))?;
        let size = data.len() as u64;

        if size > MAX_FILE_SIZE {
            return Err(AppError::PayloadTooLarge(format!(
                "file exceeds {} MB limit",
                MAX_FILE_SIZE / 1024 / 1024,
            )));
        }

        let ext = extension_for_content_type(&content_type);
        let file_uuid = Uuid::new_v4();
        let relative_path = format!("{}/{journey_id}/{file_uuid}.{ext}", auth.user_id);
        let full_path = base_path.join(&relative_path);

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&full_path, &data).await?;

        let att_id = db::attachments::Create {
            hop_id: journey_id,
            user_id: auth.user_id,
            filename: &original_filename,
            content_type: &content_type,
            size_bytes: i64::try_from(size).unwrap_or(i64::MAX),
            storage_path: &relative_path,
        }
        .execute(&state.db)
        .await?;

        if let Some(row) = (db::attachments::GetById {
            id: att_id,
            user_id: auth.user_id,
        })
        .execute(&state.db)
        .await?
        {
            created.push(AttachmentResponse::from(row));
        }
    }

    Ok((StatusCode::CREATED, Json(created)).into_response())
}

pub async fn upload_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(journey_id): Path<i64>,
    multipart: Multipart,
) -> Response {
    match upload_inner(state, auth, journey_id, multipart).await {
        Ok(resp) => resp,
        Err(err) => err.into_response(),
    }
}

pub async fn serve_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((journey_id, attachment_id)): Path<(i64, i64)>,
) -> Response {
    match serve_inner(state, auth, journey_id, attachment_id).await {
        Ok(resp) => resp,
        Err(err) => err.into_response(),
    }
}

async fn serve_inner(
    state: AppState,
    auth: AuthUser,
    journey_id: i64,
    attachment_id: i64,
) -> Result<Response, AppError> {
    let base_path = require_storage_path(&state)?;

    let row = db::attachments::GetById {
        id: attachment_id,
        user_id: auth.user_id,
    }
    .execute(&state.db)
    .await?
    .ok_or_else(|| AppError::MissingField("attachment not found"))?;

    if row.hop_id != journey_id {
        return Err(AppError::MissingField("attachment not found"));
    }

    let full_path = base_path.join(&row.storage_path);
    let data = fs::read(&full_path).await?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, row.content_type.as_str()),
            (header::CACHE_CONTROL, "private, max-age=86400"),
        ],
        Body::from(data),
    )
        .into_response())
}

/// `OpenAPI` metadata for the download attachment endpoint.
pub fn serve_handler_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Download an attachment")
        .description("Serve the raw file for a specific attachment.")
        .tag("attachments")
        .response_with::<200, (), _>(|res| res.description("File content"))
        .response_with::<404, Json<ErrorResponse>, _>(|res| res.description("Attachment not found"))
}

pub async fn delete_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path((journey_id, attachment_id)): Path<(i64, i64)>,
) -> Response {
    let format = negotiate_format(&headers);
    match delete_inner(state, auth, journey_id, attachment_id, format).await {
        Ok(resp) => resp,
        Err(err) => err.into_format_response(format),
    }
}

async fn delete_inner(
    state: AppState,
    auth: AuthUser,
    journey_id: i64,
    attachment_id: i64,
    format: ResponseFormat,
) -> Result<Response, AppError> {
    let base_path = require_storage_path(&state)?;

    let row = db::attachments::GetById {
        id: attachment_id,
        user_id: auth.user_id,
    }
    .execute(&state.db)
    .await?
    .ok_or_else(|| AppError::MissingField("attachment not found"))?;

    if row.hop_id != journey_id {
        return Err(AppError::MissingField("attachment not found"));
    }

    let full_path = base_path.join(&row.storage_path);

    db::attachments::Delete {
        id: attachment_id,
        user_id: auth.user_id,
    }
    .execute(&state.db)
    .await?;

    if full_path.exists() {
        fs::remove_file(&full_path).await?;
    }

    match format {
        ResponseFormat::Html => {
            Ok(axum::response::Redirect::to(&format!("/journeys/{journey_id}")).into_response())
        }
        _ => Ok((
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response()),
    }
}

/// `OpenAPI` metadata for the delete attachment endpoint.
pub fn delete_handler_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Delete an attachment")
        .description("Remove an attachment and its file from storage.")
        .tag("attachments")
        .response_with::<200, Json<super::StatusResponse>, _>(|res| {
            res.description("Attachment deleted")
        })
        .response_with::<404, Json<ErrorResponse>, _>(|res| res.description("Attachment not found"))
}

pub async fn list_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(journey_id): Path<i64>,
) -> Response {
    match list_inner(state, auth, journey_id).await {
        Ok(resp) => resp,
        Err(err) => err.into_response(),
    }
}

async fn list_inner(
    state: AppState,
    auth: AuthUser,
    journey_id: i64,
) -> Result<Response, AppError> {
    let rows = db::attachments::GetByHopId {
        hop_id: journey_id,
        user_id: auth.user_id,
    }
    .execute(&state.db)
    .await?;

    let response: Vec<AttachmentResponse> =
        rows.into_iter().map(AttachmentResponse::from).collect();
    Ok((StatusCode::OK, Json(response)).into_response())
}

/// `OpenAPI` metadata for the list attachments endpoint.
pub fn list_handler_docs(op: TransformOperation) -> TransformOperation {
    op.summary("List attachments for a journey")
        .description("Return all attachments associated with a journey.")
        .tag("attachments")
        .response_with::<200, Json<Vec<AttachmentResponse>>, _>(|res| {
            res.description("Attachment list")
        })
}
