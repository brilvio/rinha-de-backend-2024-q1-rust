use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    model::{SaldoModel, TransactionModel, TransactionSelectModel, TransactionsModel, UserModel},
    schema::CreateTransactionSchema,
    AppState,
};

pub async fn get_cliente_handler(
    Path(id): Path<i32>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // let db = data.db.lock().await;
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
    //let db = data.db.lock().await;
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
    // validate body
    //let db = data.db.lock().await;

    if body.tipo != "d" && body.tipo != "c" {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Tipo inválido",
        });
        return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(error_response)));
    }

    if body.tipo == "d" && body.valor > data.limites[&id] {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Valor inválido",
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

    if body.descricao.len() > 10 {
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

    let mut tx = data.db.begin().await.map_err(|e| {
        let error_response = serde_json::json!({
            "message": format!("Error starting transaction: {:?}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let query_result = sqlx::query!(
        "UPDATE users SET saldo = saldo + $1 WHERE id = $2 RETURNING saldo, limite",
        if body.tipo == "d" {
            -body.valor
        } else {
            body.valor
        },
        id
    )
    .fetch_one(&mut *tx)
    .await;

    if let Ok(user) = query_result {
        if body.tipo == "d" && ((user.saldo * -1) + body.valor) > user.limite {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Limite insuficiente",
            });
            let _ = tx.rollback().await;
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

        // Check the result
        if query_result.is_ok() {
            // The transaction was inserted successfully, so commit the transaction
            let _ = tx.commit().await;
            let transaction_response = json!({"limite": user.limite,"saldo": user.saldo});
            return Ok((StatusCode::OK, Json(transaction_response)));
        }
        let _ = tx.rollback().await;
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Error inserting transaction"})),
        ));
    }
    let error_response = serde_json::json!({
        "message": "Not Found",
    });
    let _ = tx.rollback().await;
    return Err((StatusCode::NOT_FOUND, Json(error_response)));
}

pub async fn get_extrato_handler(
    Path(id): Path<i32>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(
        TransactionSelectModel,
        "SELECT coalesce(t.valor,0) as valor, coalesce(t.tipo,'x') as tipo, coalesce(t.descricao,'none') as descricao, t.realizada_em, u.saldo, u.limite FROM users u 
        LEFT JOIN transactions t ON 
        u.id = t.user_id 
        WHERE u.id = $1 
        ORDER BY t.realizada_em DESC  
        LIMIT 10",
        id
    )
    .fetch_all(&data.db)
    .await;

    if query_result.is_err() {
        println!("{:?}", query_result.unwrap_err());
        let error_response = serde_json::json!({
            "message": "Not Found",
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let records = query_result.unwrap();

    if records.is_empty() {
        let error_response = serde_json::json!({
            "message": "Not Found",
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }
    let user = &records[0];
    let mut transactions = vec![];

    if records[0].descricao != Some("none".to_string()) && records[0].tipo != Some("x".to_string())
    {
        transactions = records
            .clone()
            .into_iter()
            .map(|record| TransactionModel {
                valor: record.valor,
                tipo: record.tipo,
                descricao: record.descricao,
                realizada_em: record.realizada_em,
            })
            .collect::<Vec<TransactionModel>>();
    }

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
