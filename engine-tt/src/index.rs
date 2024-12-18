use std::sync::{Arc, Mutex};

use anyhow::Error;
use tantivy::{
    directory::MmapDirectory,
    doc,
    schema::{Field, Schema, FAST, STORED, TEXT},
    Index, IndexReader, IndexWriter,
};

use crate::docs::Doc;

pub struct MainIndexField {
    pub url: Field,
    pub title: Field,
    pub body: Field,
    pub uuid: Field,
}

pub struct MainIndexPack {
    pub index: Index,
    pub field: MainIndexField,
    pub writer: Mutex<IndexWriter>,
    pub reader: IndexReader,
}

impl MainIndexPack {
    pub fn load_default() -> Result<Arc<MainIndexPack>, Error> {
        let index_path = "./index";

        let mut schema_builder = Schema::builder();
        let url = schema_builder.add_text_field("url", STORED);
        let title = schema_builder.add_text_field("title", TEXT | STORED);
        let body = schema_builder.add_text_field("body", TEXT | STORED);
        let uuid = schema_builder.add_text_field("uuid", STORED);

        let schema = schema_builder.build();

        std::fs::create_dir_all(index_path)?;
        let dir = MmapDirectory::open(index_path)?;

        let index = Index::open_or_create(dir, schema.clone())?;

        Ok(Arc::new(MainIndexPack {
            writer: Mutex::new(index.writer(100_000_000)?),
            reader: index.reader()?,
            index,
            field: MainIndexField {
                url,
                title,
                body,
                uuid,
            },
        }))
    }
}

pub struct PageIndexField {
    pub body: Field,
    pub uuid: Field,
}

pub struct PageIndexPack {
    pub index: Index,
    pub field: PageIndexField,
    pub writer: Mutex<IndexWriter>,
    pub reader: IndexReader,
}

impl PageIndexPack {
    pub fn load_default() -> tantivy::Result<Arc<PageIndexPack>> {
        let mut schema_builder = Schema::builder();

        // let url = schema_builder.add_text_field("url", STORED);
        // let title = schema_builder.add_text_field("title", TEXT | STORED);
        let body = schema_builder.add_text_field("body", TEXT | STORED);
        let uuid = schema_builder.add_text_field("uuid", TEXT | STORED | FAST);

        let schema = schema_builder.build();

        let index_path = "./page-index";
        std::fs::create_dir_all(&index_path)?;
        let dir = MmapDirectory::open(&index_path)?;

        let index = Index::open_or_create(dir, schema.clone())?;

        Ok(Arc::new(PageIndexPack {
            writer: Mutex::new(index.writer(100_000_000)?),
            reader: index.reader()?,
            index,
            field: PageIndexField { body, uuid },
        }))
    }

    pub fn generate_page_index(&self, page: &Doc, page_uuid: &str) -> Result<(), Error> {
        let PageIndexPack { field, writer, .. } = self;

        let mut writer = writer.lock().unwrap();

        for idx in 0..(page.body.len()) {
            let mut cur_idx = idx;
            let mut total_len = 0;

            let mut doc = doc!();
            // doc.add_text(field.url, &page.url);
            // doc.add_text(field.title, &page.title);

            while (total_len < 300) && (cur_idx < page.body.len()) {
                total_len += page.body[cur_idx].len();
                doc.add_text(field.body, &page.body[cur_idx]);
                cur_idx += 1;
            }
            doc.add_text(field.uuid, page_uuid);

            writer.add_document(doc)?;
        }
        writer.commit()?;

        Ok(())
    }
}
