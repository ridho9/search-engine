use std::sync::Arc;

use anyhow::Error;
use serde::Serialize;
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Field, Schema, Value, FAST, STORED, TEXT},
    Index, IndexWriter, TantivyDocument,
};

use crate::{index::IndexField, ServerConfig};

#[derive(Serialize)]
pub struct HitsItem {
    score: f32,
    doc: HitsDoc,
    relevant_body: Vec<String>,
}

#[derive(Serialize)]
pub struct HitsDoc {
    url: Vec<String>,
    title: Vec<String>,
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

        let hits_doc = HitsDoc {
            url: url.iter().map(|&s| s.to_owned()).collect(),
            title: title.iter().map(|&s| s.to_owned()).collect(),
            body: body.iter().map(|&s| s.to_owned()).collect(),
        };

        // TODO: use mmap index
        let relevant_body = get_relevant_body(&hits_doc, &query_param)?;
        // let relevant_body = vec![];

        let item = HitsItem {
            score,
            doc: HitsDoc {
                body: vec![],
                ..hits_doc
            },
            relevant_body,
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

fn build_page_index(page: &HitsDoc) -> tantivy::Result<(Index, IndexField)> {
    let mut schema_builder = Schema::builder();

    let url = schema_builder.add_text_field("url", STORED);
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let body = schema_builder.add_text_field("body", TEXT | STORED | FAST);

    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema.clone());

    let mut writer: IndexWriter = index.writer(15_000_000)?;

    for idx in 0..(page.body.len()) {
        let mut cur_idx = idx;
        let mut total_len = 0;

        let mut doc = doc!();
        doc.add_text(url, &page.url[0]);
        doc.add_text(title, &page.title[0]);

        while (total_len < 500) && (cur_idx < page.body.len()) {
            // cumulated_lines.push(lines[cur_idx]);
            total_len += page.body[cur_idx].len();

            doc.add_text(body, &page.body[cur_idx]);

            cur_idx += 1;
        }
        // println!("{:?}", cumulated_lines);

        writer.add_document(doc)?;
    }
    writer.commit()?;

    Ok((index, IndexField { url, title, body }))
}

fn get_relevant_body(page: &HitsDoc, query_str: &str) -> Result<Vec<String>, Error> {
    let (index, field) = build_page_index(page)?;

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
