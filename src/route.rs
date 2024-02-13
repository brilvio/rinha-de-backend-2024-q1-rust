use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handler::{
        create_transaction_handler, get_cliente_handler, get_clientes_list_handler,
        get_extrato_handler,
    },
    AppState,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/clientes/:id/transacoes", post(create_transaction_handler))
        .route("/clientes/:id/extrato", get(get_extrato_handler))
        .route("/clientes/:id", get(get_cliente_handler))
        .route("/clientes/", get(get_clientes_list_handler))
        .with_state(app_state)
}
