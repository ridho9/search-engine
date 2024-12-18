use anyhow::Error;
use tantivy::{
    directory::MmapDirectory,
    doc,
    schema::{Field, Schema, FAST, STORED, TEXT},
    Index, IndexWriter,
};

use crate::docs::Doc;

pub struct IndexField {
    pub url: Field,
    pub title: Field,
    pub body: Field,
    pub uuid: Field,
}

pub fn get_index() -> tantivy::Result<(Index, IndexField)> {
    let index_path = "./index";

    let mut schema_builder = Schema::builder();
    let url = schema_builder.add_text_field("url", STORED);
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let body = schema_builder.add_text_field("body", TEXT | STORED | FAST);
    let uuid = schema_builder.add_text_field("uuid", STORED);

    let schema = schema_builder.build();

    std::fs::create_dir_all(index_path)?;
    let dir = MmapDirectory::open(index_path)?;

    let index = Index::open_or_create(dir, schema.clone())?;
    let field = IndexField {
        url,
        title,
        body,
        uuid,
    };

    Ok((index, field))
}

pub fn get_page_index(page_uuid: &str) -> tantivy::Result<(Index, IndexField)> {
    let mut schema_builder = Schema::builder();

    let url = schema_builder.add_text_field("url", STORED);
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let body = schema_builder.add_text_field("body", TEXT | STORED | FAST);
    let uuid = schema_builder.add_text_field("uuid", STORED);

    let schema = schema_builder.build();

    let index_path = format!("./page-index/{:}", page_uuid);
    std::fs::create_dir_all(&index_path)?;
    let dir = MmapDirectory::open(&index_path)?;

    let index = Index::open_or_create(dir, schema.clone())?;

    Ok((
        index,
        IndexField {
            url,
            title,
            body,
            uuid,
        },
    ))
}

pub fn generate_page_index(page: &Doc, page_uuid: &str) -> Result<(), Error> {
    let (index, field) = get_page_index(page_uuid)?;
    let mut writer: IndexWriter = index.writer(15_000_000)?;

    for idx in 0..(page.body.len()) {
        let mut cur_idx = idx;
        let mut total_len = 0;

        let mut doc = doc!();
        doc.add_text(field.url, &page.url);
        doc.add_text(field.title, &page.title);

        while (total_len < 300) && (cur_idx < page.body.len()) {
            total_len += page.body[cur_idx].len();
            doc.add_text(field.body, &page.body[cur_idx]);
            cur_idx += 1;
        }

        writer.add_document(doc)?;
    }
    writer.commit()?;

    Ok(())
}
