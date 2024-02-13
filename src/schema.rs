use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct ParamOptions {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTransactionSchema {
    pub valor: i64,
    pub tipo: String,
    pub descricao: String,
}
