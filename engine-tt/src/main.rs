use tantivy::{
    schema::{Schema, STORED, TEXT},
    Index, IndexWriter,
};

fn main() -> tantivy::Result<()> {
    let index_path = "./index";

    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("text", TEXT);
    schema_builder.add_text_field("url", TEXT);
    schema_builder.add_text_field("meta_title", TEXT);
    schema_builder.add_text_field("meta_desc", TEXT);

    let schema = schema_builder.build();

    let index = Index::create_in_dir(&index_path, schema.clone())?;

    let mut index_writer: IndexWriter = index.writer(50_000_000)?;

    index_writer.commit()?;

    Ok(())
}
