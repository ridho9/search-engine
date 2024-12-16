mod index;

use std::sync::{Arc, Mutex};

use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use index::{build_index, IndexField};
use serde::{Deserialize, Serialize};
use tantivy::{
    collector::TopDocs, doc, query::QueryParser, schema::Value, Document, Index, IndexReader,
    IndexWriter, TantivyDocument,
};

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
        writer: Mutex::new(index.writer(100_000_000)?),
        reader: index.reader()?,
        field,
        index,
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/api/docs", post(insert_doc))
        .route("/api/docs", delete(delete_docs))
        .route("/api/docs", get(query_docs))
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
    documents: Vec<Doc>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Doc {
    // id: String,
    url: String,
    title: String,
    body: String,
}

async fn insert_doc(
    State(state): State<Arc<ServerConfig>>,
    Json(payload): Json<InsertDoc>,
) -> Result<String, AppError> {
    let mut writer = state.writer.lock().unwrap();

    let mut len = 0;

    for d in payload.documents {
        println!("insert {:#?} ", d.url);
        writer.add_document(doc!(
            state.field.url => d.url,
            state.field.title => d.title,
            state.field.body => d.body,
        ))?;
        len += 1;
    }

    let commit_res = writer.commit()?;

    Ok(format!("insert {} items", len))
}

async fn delete_docs(State(state): State<Arc<ServerConfig>>) -> Result<String, AppError> {
    println!("deleting docs");

    let mut writer = state.writer.lock().unwrap();
    let del_res = writer.delete_all_documents()?;
    let commit_res = writer.commit()?;

    return Ok(format!("del res {} commit res {}", del_res, commit_res));
}

#[derive(Deserialize)]
struct QueryParam {
    query: String,
}

async fn query_docs(
    State(state): State<Arc<ServerConfig>>,
    Query(query): Query<QueryParam>,
) -> Result<impl IntoResponse, AppError> {
    let reader = &state.reader;
    let field = &state.field;
    let index = &state.index;

    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![field.title, field.body]);
    let query = query_parser.parse_query(&query.query)?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    let mut ret_docs = vec![];
    for (_score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
        let j = retrieved_doc.to_json(&index.schema());
        ret_docs.push(j);
    }
    let joined_str = format!("[{}]", ret_docs.join(", "));

    Ok(([(header::CONTENT_TYPE, "application/json")], joined_str))
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
