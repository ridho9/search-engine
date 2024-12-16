mod index;

use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use index::{build_index, IndexField};
use serde::Deserialize;
use tantivy::{doc, Index, IndexReader, IndexWriter};

struct ServerConfig {
    index: Index,
    field: IndexField,
    writer: Mutex<IndexWriter>,
    reader: IndexReader,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let (index, field) = build_index().expect("Failed building index");
    println!("{:#?}\n", index);

    let server_config = Arc::new(ServerConfig {
        writer: Mutex::new(index.writer(50_000_000)?),
        reader: index.reader_builder().try_into()?,
        field,
        index,
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/api/docs", post(insert_doc))
        .route("/api/docs", delete(delete_docs))
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
    // id: String,
    url: String,
    title: String,
    body: String,
}

async fn insert_doc(
    State(state): State<Arc<ServerConfig>>,
    Json(payload): Json<InsertDoc>,
) -> Result<String, AppError> {
    println!("insert {:#?} ", payload.url);

    let mut writer = state.writer.lock().unwrap();
    writer.add_document(doc!(
        state.field.url => payload.url,
        state.field.title => payload.title,
        state.field.body => payload.body,
    ))?;

    let commit_res = writer.commit()?;

    Ok(format!("insert commit res {}", commit_res))
}

async fn delete_docs(State(state): State<Arc<ServerConfig>>) -> Result<String, AppError> {
    println!("deleting docs");

    let mut writer = state.writer.lock().unwrap();
    let del_res = writer.delete_all_documents()?;
    let commit_res = writer.commit()?;

    return Ok(format!("del res {} commit res {}", del_res, commit_res));
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
