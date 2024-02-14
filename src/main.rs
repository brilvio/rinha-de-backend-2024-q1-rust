mod handler;
mod model;
mod route;
mod schema;

use std::{collections::HashMap, sync::Arc};

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use dotenv::dotenv;
use route::create_router;

use tower_http::cors::CorsLayer;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub struct AppState {
    db: Pool<Postgres>,
    limites: HashMap<i32, i64>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = match PgPoolOptions::new()
        // .after_connect(|conn, _| {
        //     Box::pin(async move {
        //         conn.execute("SET default_transaction_isolation TO 'REPEATABLE READ'")
        //             .await?;
        //         Ok(())
        //     })
        // })
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    sqlx::migrate!().run(&pool).await.unwrap();

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:9999".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let mut limites = HashMap::new();
    let clients = sqlx::query!("SELECT * FROM users")
        .fetch_all(&pool)
        .await
        .unwrap();

    for cliente in clients {
        limites.insert(cliente.id as i32, cliente.limite as i64);
    }

    let app = create_router(Arc::new(AppState {
        db: pool.clone(),
        limites,
    }))
    .layer(cors);

    println!("🚀 Server started successfully");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
