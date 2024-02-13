use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    model::{SaldoModel, TransactionModel, TransactionsModel, UserModel},
    schema::CreateTransactionSchema,
    AppState,
};

pub async fn get_cliente_handler(
    Path(id): Path<i32>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(
        UserModel,
        "SELECT * FROM users where id = $1 ORDER by id",
        id
    )
    .fetch_all(&data.db)
    .await;

    if query_result.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Something bad happened while fetching all note items",
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let clientes = query_result.unwrap();

    let json_response = serde_json::json!(clientes);
    Ok(Json(json_response))
}

pub async fn get_clientes_list_handler(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(UserModel, "SELECT * FROM users ORDER by id",)
        .fetch_all(&data.db)
        .await;

    if query_result.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Something bad happened while fetching all note items",
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let clientes = query_result.unwrap();

    let json_response = serde_json::json!(clientes);
    Ok(Json(json_response))
}

pub async fn create_transaction_handler(
    Path(id): Path<i32>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateTransactionSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut tx = data.db.begin().await.map_err(|e| {
        let error_response = serde_json::json!({
            "message": format!("Error starting transaction: {:?}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    // validate body
    if body.tipo != "d" && body.tipo != "c" {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Tipo inválido",
        });
        return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(error_response)));
    }

    if body.descricao == "" {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Descrição inválida",
        });
        return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(error_response)));
    }

    if body.valor <= 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Valor inválido",
        });
        return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(error_response)));
    }

    let query_result = sqlx::query!("SELECT saldo, limite FROM users WHERE id = $1", id)
        .fetch_one(&mut *tx)
        .await;

    if query_result.is_err() {
        let error_response = serde_json::json!({
            "message": "Not Found",
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let mut user = query_result.unwrap();

    if body.tipo == "d" && ((user.saldo * -1) + body.valor) > user.limite {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Limite insuficiente",
        });
        return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(error_response)));
    }

    let query_result = sqlx::query!(
        "INSERT INTO transactions (user_id, valor, tipo, descricao) VALUES ($1, $2, $3, $4)",
        id,
        body.valor,
        body.tipo,
        body.descricao
    )
    .execute(&mut *tx)
    .await;

    if query_result.is_err() {
        let error_response = serde_json::json!({
            "message": "Error inserting in transactions",
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    user.saldo = if body.tipo == "d" {
        user.saldo - body.valor
    } else {
        user.saldo + body.valor
    };

    let query_result = sqlx::query!("UPDATE users SET saldo = $1 WHERE id = $2", user.saldo, id)
        .execute(&mut *tx)
        .await;

    match query_result {
        Ok(_) => {
            let transaction_response = json!({"limite": user.limite,"saldo": user.saldo});
            tx.commit().await.map_err(|e| {
                let error_response = serde_json::json!({
                    "message": format!("Error committing transaction: {:?}", e),
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;

            return Ok((StatusCode::OK, Json(transaction_response)));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error updating saldo","message": format!("{:?}", e)})),
            ));
        }
    }
}

pub async fn get_extrato_handler(
    Path(id): Path<i32>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query!(
        "SELECT t.valor, t.tipo, t.descricao, t.realizada_em FROM transactions t WHERE t.user_id = $1 ORDER BY t.realizada_em DESC limit 10",
        id
    )
    .fetch_all(&data.db)
    .await;

    if query_result.is_err() {
        let error_response = serde_json::json!({
            "message": "Not Found",
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let transactions = query_result.unwrap();

    let query_result = sqlx::query!("SELECT saldo, limite FROM users WHERE id = $1", id)
        .fetch_one(&data.db)
        .await;

    if query_result.is_err() {
        let error_response = serde_json::json!({
            "message": "Not Found",
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let user = query_result.unwrap();

    let transactions = transactions
        .into_iter()
        .map(|record| TransactionModel {
            valor: record.valor,
            tipo: record.tipo,
            descricao: record.descricao,
            realizada_em: record.realizada_em,
        })
        .collect::<Vec<TransactionModel>>();

    let transaction = TransactionsModel {
        saldo: SaldoModel {
            saldo: user.saldo,
            limite: user.limite,
            data_extrato: chrono::offset::Utc::now(),
        },
        ultimas_transacoes: transactions,
    };

    let json_response = serde_json::json!(transaction);
    Ok(Json(json_response))
}
