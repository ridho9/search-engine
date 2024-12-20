mod docs;
mod index;

use std::{sync::Arc, time::Instant};

use axum::{
    extract::{Query, State},
    http::{HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use docs::{query_docs, Doc, HitsItem};
use index::{MainIndexPack, PageIndexPack};
use serde::{Deserialize, Serialize};
use tantivy::doc;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[derive(Clone)]
struct ServerConfig {
    main_index: Arc<MainIndexPack>,
    page_index: Arc<PageIndexPack>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let index_pack = MainIndexPack::load_default().expect("Failed loading main index");
    let page_index_pack = PageIndexPack::load_default().expect("Failed loading page index");
    println!("{:#?}\n", index_pack.index);

    let server_config = ServerConfig {
        main_index: index_pack,
        page_index: page_index_pack,
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/api/docs", post(insert_doc))
        .route("/api/docs", delete(delete_docs))
        .route("/api/docs", get(req_query_docs))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3001".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST]),
        )
        .with_state(server_config);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn root() -> &'static str {
    "Server running"
}

#[derive(Deserialize, Debug)]
struct InsertDoc {
    documents: Vec<Doc>,
}

async fn insert_doc(
    State(state): State<ServerConfig>,
    Json(payload): Json<InsertDoc>,
) -> Result<String, AppError> {
    let mut writer = state.main_index.writer.lock().unwrap();
    let len = payload.documents.len();

    for d in payload.documents {
        println!("insert {:#?} ", d.url);

        let uuid = Uuid::new_v4().to_string();

        let mut doc = doc!();
        doc.add_text(state.main_index.field.url, &d.url);
        doc.add_text(state.main_index.field.title, &d.title);
        doc.add_text(state.main_index.field.uuid, &uuid);
        for b in &d.body {
            doc.add_text(state.main_index.field.body, b);
        }

        writer.add_document(doc)?;

        state.page_index.stage_page_index(&d, &uuid)?;
    }
    state.page_index.writer.lock().unwrap().commit()?;

    writer.commit()?;

    Ok(format!("insert {} items", len))
}

async fn delete_docs(State(state): State<ServerConfig>) -> Result<String, AppError> {
    let mut writer = state.main_index.writer.lock().unwrap();
    let del_res = writer.delete_all_documents()?;
    let commit_res = writer.commit()?;

    return Ok(format!("del res {} commit res {}", del_res, commit_res));
}

#[derive(Deserialize)]
struct QueryParam {
    query: String,
}

#[derive(Serialize)]
struct QueryResponse {
    q: String,
    elapsed_ms: f64,
    hits: Vec<HitsItem>,
    count: usize,
}

async fn req_query_docs(
    State(state): State<ServerConfig>,
    Query(query_param): Query<QueryParam>,
) -> Result<impl IntoResponse, AppError> {
    let start = Instant::now();

    let (ret_hits, count) = query_docs(&state, &query_param.query)?;

    let elapsed_nanos = start.elapsed().as_nanos();
    let elapsed_milis = (elapsed_nanos as f64) / 1_000_000.0;

    let resp = QueryResponse {
        q: query_param.query,
        elapsed_ms: elapsed_milis,
        hits: ret_hits,
        count,
    };

    Ok(Json(resp))
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
