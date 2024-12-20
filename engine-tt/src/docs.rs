use std::sync::Arc;

use anyhow::Error;
use serde::{Deserialize, Serialize};
use tantivy::{
    collector::{Count, TopDocs},
    doc,
    query::QueryParser,
    schema::{Field, Schema, Value, FAST, STORED, TEXT},
    Index, IndexWriter, TantivyDocument,
};

use crate::{
    index::{MainIndexField, PageIndexField, PageIndexPack},
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

pub fn query_docs(
    state: &ServerConfig,
    query_param: &str,
) -> Result<(Vec<HitsItem>, usize), Error> {
    let reader = &state.main_index.reader;
    let field = &state.main_index.field;
    let index = &state.main_index.index;
    let searcher = reader.searcher();
    let mut query_parser = QueryParser::for_index(&index, vec![field.title, field.body]);
    query_parser.set_field_boost(field.title, 2.0);

    let query = query_parser.parse_query(&query_param)?;
    let (top_docs, total_count) = searcher.search(&query, &(TopDocs::with_limit(10), Count))?;
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
        let relevant_body = get_relevant_body(&state.page_index, &uuid, &query_param)?;
        // let relevant_body = vec![];

        let item = HitsItem {
            score,
            doc: hits_doc,
            relevant_body,
            uuid,
        };

        ret_hits.push(item);
    }
    Ok((ret_hits, total_count))
}

fn retrieve_str_fields(retrieved_doc: &TantivyDocument, field: Field) -> Vec<&str> {
    let ret_url: Vec<_> = retrieved_doc
        .get_all(field)
        .map(|v| v.as_str().unwrap())
        .collect();
    return ret_url;
}

fn get_relevant_body(
    page_index: &PageIndexPack,
    page_uuid: &str,
    query_str: &str,
) -> Result<Vec<String>, Error> {
    let index = &page_index.index;
    let reader = &page_index.reader;
    let field = &page_index.field;

    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![field.body]);
    let new_query_str = format!("{} {}", format!(r#"uuid:"{}""#, page_uuid), query_str);
    let query = query_parser.parse_query(&new_query_str)?;
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
