use std::sync::Arc;

use anyhow::Error;
use serde::Serialize;
use tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::{Field, Value},
    TantivyDocument,
};

use crate::ServerConfig;

#[derive(Serialize)]
pub struct HitsItem {
    score: f32,
    doc: HitsDoc,
}

#[derive(Serialize)]
pub struct HitsDoc {
    url: Vec<String>,
    title: Vec<String>,
    // body: Vec<String>,
    body: Vec<String>,
}

pub fn query_docs(state: Arc<ServerConfig>, query_param: &str) -> Result<Vec<HitsItem>, Error> {
    let reader = &state.reader;
    let field = &state.field;
    let index = &state.index;
    let searcher = reader.searcher();
    let mut query_parser = QueryParser::for_index(&index, vec![field.title, field.body]);
    query_parser.set_field_boost(field.title, 2.0);
    let query = query_parser.parse_query(&query_param)?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    let mut ret_hits = vec![];
    for (score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

        let url = retrieve_str_fields(&retrieved_doc, field.url);
        let title = retrieve_str_fields(&retrieved_doc, field.title);
        let body = retrieve_str_fields(&retrieved_doc, field.body);

        let item = HitsItem {
            score,
            doc: HitsDoc { url, title, body },
        };

        ret_hits.push(item);
    }
    Ok(ret_hits)
}

fn retrieve_str_fields(retrieved_doc: &TantivyDocument, field: Field) -> Vec<String> {
    let ret_url: Vec<_> = retrieved_doc
        .get_all(field)
        .map(|v| v.as_str().unwrap().to_owned())
        .collect();
    return ret_url;
}
