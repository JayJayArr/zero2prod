use anyhow::anyhow;
use axum::{
    body::{Body, to_bytes},
    response::Response,
};
use reqwest::StatusCode;
use sqlx::Executor;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::idempotency::IdempotencyKey;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

pub enum NextAction {
    StartProcessing(Transaction<'static, Postgres>),
    ReturnSavedResponse(Response),
}

pub async fn get_saved_response(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<Option<Response>, anyhow::Error> {
    let saved_response = sqlx::query!(
        r#"
    SELECT 
        response_status_code as "response_status_code!",
        response_headers as "response_headers!: Vec<HeaderPairRecord>",
        response_body as "response_body!"
    FROM
        idempotency
    WHERE user_id = $1 AND
    idempotency_key = $2
    "#,
        user_id,
        idempotency_key.as_ref()
    )
    .fetch_optional(pool)
    .await?;

    if let Some(r) = saved_response {
        let status_code = StatusCode::from_u16(r.response_status_code.try_into()?)?;
        let mut response = Response::builder().status(status_code);
        for HeaderPairRecord { name, value } in r.response_headers {
            response = response.header(name, value);
        }
        Ok(Some(response.body(r.response_body.into())?))
    } else {
        Ok(None)
    }
}

pub async fn save_response(
    mut transaction: Transaction<'static, Postgres>,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
    http_response: Response,
) -> Result<Response, anyhow::Error> {
    let (parts, body) = http_response.into_parts();
    let body = to_bytes(body, usize::MAX)
        .await
        .map_err(|e| anyhow!("{}", e))?;
    let status_code = parts.status.as_u16() as i16;
    let headers = {
        let mut h = Vec::with_capacity(parts.headers.len());
        for (name, value) in parts.headers.iter() {
            let name = name.as_str().to_owned();
            let value = value.as_bytes().to_owned();
            h.push(HeaderPairRecord { name, value });
        }
        h
    };

    let query = sqlx::query_unchecked!(
        r#"
        UPDATE idempotency
        SET
            response_status_code = $3,
            response_headers = $4,
            response_body = $5
        WHERE 
            user_id = $1 AND
            idempotency_key = $2
        "#,
        user_id,
        idempotency_key.as_ref(),
        status_code,
        headers,
        body.as_ref()
    );
    transaction.execute(query).await?;
    transaction.commit().await?;

    let body = Body::from(body);
    let http_response = Response::from_parts(parts, body);
    Ok(http_response)
}

pub async fn try_processing(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<NextAction, anyhow::Error> {
    let mut transaction = pool.begin().await?;
    let query = sqlx::query!(
        r#"
    INSERT INTO idempotency (
        user_id,
        idempotency_key,
        created_at
    )
    VALUES ($1, $2, now())
    ON CONFLICT DO NOTHING
    "#,
        user_id,
        idempotency_key.as_ref()
    );
    let n_inserted_rows = transaction.execute(query).await?.rows_affected();

    if n_inserted_rows > 0 {
        Ok(NextAction::StartProcessing(transaction))
    } else {
        let saved_response = get_saved_response(pool, idempotency_key, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("We expected a saved response, we didn't find it."))?;
        Ok(NextAction::ReturnSavedResponse(saved_response))
    }
}
