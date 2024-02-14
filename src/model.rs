use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct UserModel {
    pub id: i32,
    pub nome: String,
    pub limite: i64,
    pub saldo: i64,
    #[serde(rename = "createdAt")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct SaldoModel {
    #[serde(rename = "total")]
    pub saldo: i64,
    pub limite: i64,
    pub data_extrato: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct TransactionModel {
    pub valor: Option<i64>,
    pub tipo: Option<String>,
    pub descricao: Option<String>,
    pub realizada_em: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionsModel {
    pub saldo: SaldoModel,
    pub ultimas_transacoes: Vec<TransactionModel>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionSelectModel {
    pub valor: Option<i64>,
    pub tipo: Option<String>,
    pub descricao: Option<String>,
    pub realizada_em: Option<chrono::DateTime<chrono::Utc>>,
    pub saldo: i64,
    pub limite: i64,
}
