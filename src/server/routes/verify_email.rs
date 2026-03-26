use crate::{
    db,
    server::{AppState, session::sha256_hex},
};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct VerifyQuery {
    token: String,
}

pub async fn handler(State(state): State<AppState>, Query(query): Query<VerifyQuery>) -> Response {
    let token_hash = sha256_hex(&query.token);

    let row = match (db::email_verifications::GetByTokenHash {
        token_hash: &token_hash,
    })
    .execute(&state.db)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Redirect::to("/settings?error=Invalid+or+expired+verification+link")
                .into_response();
        }
        Err(err) => {
            tracing::error!(error = %err, "failed to look up verification token");
            return Redirect::to("/settings?error=Verification+failed").into_response();
        }
    };

    let expired = sqlx::query_scalar::<_, bool>("SELECT ? < datetime('now')")
        .bind(&row.expires_at)
        .fetch_one(&state.db)
        .await
        .unwrap_or(true);

    if expired {
        let _ = (db::email_verifications::DeleteByUserId {
            user_id: row.user_id,
        })
        .execute(&state.db)
        .await;
        return Redirect::to("/settings?error=Verification+link+has+expired").into_response();
    }

    if let Err(err) = (db::users::SetEmailVerified {
        user_id: row.user_id,
    })
    .execute(&state.db)
    .await
    {
        tracing::error!(user_id = row.user_id, error = %err, "failed to mark email verified");
        return Redirect::to("/settings?error=Verification+failed").into_response();
    }

    let _ = (db::email_verifications::DeleteByUserId {
        user_id: row.user_id,
    })
    .execute(&state.db)
    .await;

    Redirect::to("/settings?email=verified").into_response()
}
