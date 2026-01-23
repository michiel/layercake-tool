## Project documents, RAG and agentic context

- The project currently has the concept of "Data sources", these will be renamed to "Data sets" and are separate from "Data sources" mentioned in this document. This renaming operation is a prerequisite for starting this task. As part of the design assume that the current "Data source" entities have been renamed to "Data sets"
- This is to be a separate crate in the layercake project workspace
- New project section "Data acquisition"
  - Subsection : "Source ingestion"
  - Subsection : "Knowledge base"
  - Subsection : "Data set creation"
- Project will have a new entity Tag, and files, datasets, and individual nodes and edges of Graph entities will be able to have zero, one or more tags (example: 'storage', 'compliance', 'iso27001')

### Workflow

- On "Source management" : User to upload files, these files are stored in the database (sea-orm managed sqlite BLOB). Files can be tagged
- Files to be processed into a vector database for context generation by LLM agents on a per-project basis
- The vector database is also in the same project database (sea-orm managed sqlite)
- Initial filestypes for ingestion: text, markdown, csv, pdf, odf, ods, xlsx, docx
- The user can select files for addition, updating, etc into the vector database
- The "Knowledge base" page allows us to manage aspects of the vector database relevant to the project (including clearing, reinitializing, etc)
- The "Data set creation" page has prompts that can be used by agents with the vector database using RAG to create layercake datasets. These are then made available in the project "Data sets" and can be used in the project plan DAG

The project uses rig.rs for agentic, rag and llm interaction. The rig-core crate is already in use in the project. Do not duplicate functionality, reuse, consolidate and refactor as necessary. An example for reference is,


```rust
use rig::providers::openai;
use rig::vector_store::in_memory_store::InMemoryVectorStore;
use rig::vector_store::VectorStore;
use rig::embeddings::EmbeddingsBuilder;
use rig::cli_chatbot::cli_chatbot;  // Import the cli_chatbot function
use std::path::Path;
use anyhow::{Result, Context};
use pdf_extract::extract_text;

fn load_pdf_content<P: AsRef<Path>>(file_path: P) -> Result<String> {
    extract_text(file_path.as_ref())
        .with_context(|| format!("Failed to extract text from PDF: {:?}", file_path.as_ref()))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize OpenAI client
    let openai_client = openai::Client::from_env();
    let embedding_model = openai_client.embedding_model("text-embedding-ada-002");

    // Create vector store
    let mut vector_store = InMemoryVectorStore::default();

    // Get the current directory and construct paths to PDF files
    let current_dir = std::env::current_dir()?;
    let documents_dir = current_dir.join("documents");

    let pdf1_path = documents_dir.join("Moores_Law_for_Everything.pdf");
    let pdf2_path = documents_dir.join("The_Last_Question.pdf");

    // Load PDF documents
    let pdf1_content = load_pdf_content(&pdf1_path)?;
    let pdf2_content = load_pdf_content(&pdf2_path)?;

    // Create embeddings and add to vector store
    let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
        .simple_document("Moores_Law_for_Everything", &pdf1_content)
        .simple_document("The_Last_Question", &pdf2_content)
        .build()
        .await?;

    vector_store.add_documents(embeddings).await?;

    // Create RAG agent
    let rag_agent = openai_client.context_rag_agent("gpt-3.5-turbo")
        .preamble("You are a helpful assistant that answers questions based on the given context from PDF documents.")
        .dynamic_context(2, vector_store.index(embedding_model))
        .build();

    // Use the cli_chatbot function to create the CLI interface
    cli_chatbot(rag_agent).await?;

    Ok(())
}
```

The vector database is subsequently available to all agents in the project.

