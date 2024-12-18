use std::sync::Arc;

use anyhow::Error;
use serde::{Deserialize, Serialize};
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Field, Schema, Value, FAST, STORED, TEXT},
    Index, IndexWriter, TantivyDocument,
};

use crate::{
    index::{get_page_index, IndexField},
    ServerConfig,
};

#[derive(Serialize)]
pub struct HitsItem {
    score: f32,
    doc: Doc,
    uuid: String,
    relevant_body: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Doc {
    pub url: String,
    pub title: String,
    pub body: Vec<String>,
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
        // let body = retrieve_str_fields(&retrieved_doc, field.body);
        let uuid = retrieve_str_fields(&retrieved_doc, field.uuid)
            .first()
            .unwrap()
            .to_string();

        let hits_doc = Doc {
            url: url.first().unwrap().to_string(),
            title: title.first().unwrap().to_string(),
            body: vec![],
        };

        // TODO: use mmap index
        let relevant_body = get_relevant_body(&uuid, &query_param)?;
        // let relevant_body = vec![];

        let item = HitsItem {
            score,
            doc: hits_doc,
            relevant_body,
            uuid,
        };

        ret_hits.push(item);
    }
    Ok(ret_hits)
}

fn retrieve_str_fields(retrieved_doc: &TantivyDocument, field: Field) -> Vec<&str> {
    let ret_url: Vec<_> = retrieved_doc
        .get_all(field)
        .map(|v| v.as_str().unwrap())
        .collect();
    return ret_url;
}

fn get_relevant_body(page_uuid: &str, query_str: &str) -> Result<Vec<String>, Error> {
    let (index, field) = get_page_index(page_uuid)?;

    let reader = index.reader()?;
    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![field.title, field.body]);
    let query = query_parser.parse_query(&query_str)?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(1))?;

    let mut ret_body = vec![];

    for (_score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
        ret_body = retrieve_str_fields(&retrieved_doc, field.body)
            .iter()
            .map(|&s| s.to_owned())
            .collect();
    }

    Ok(ret_body)
}
