use tantivy::{
    directory::MmapDirectory,
    schema::{Field, Schema, STORED, TEXT},
    Index,
};

pub struct IndexField {
    pub url: Field,
    pub title: Field,
    pub body: Field,
}

pub fn build_index() -> tantivy::Result<(Index, IndexField)> {
    let index_path = "./index";

    let mut schema_builder = Schema::builder();
    let url = schema_builder.add_text_field("url", STORED);
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let body = schema_builder.add_text_field("body", TEXT | STORED);

    let schema = schema_builder.build();

    std::fs::create_dir_all(index_path)?;
    let dir = MmapDirectory::open(index_path)?;

    let index = Index::open_or_create(dir, schema.clone())?;
    let field = IndexField { url, title, body };

    Ok((index, field))
}
