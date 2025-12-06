//! Data acquisition crate providing file ingestion, tagging, knowledge base, and
//! dataset generation services for Layercake projects.

pub mod config;
pub mod dataset_generation;
pub mod dataset_schema;
pub mod embeddings;
pub mod entities;
pub mod errors;
pub mod ingestion;
pub mod services;
pub mod tagging;
pub mod vector_store;

pub use config::{DataAcquisitionConfig, EmbeddingProviderConfig};
pub use services::{
    DataAcquisitionService, FileIngestionRequest, KnowledgeBaseCommand, KnowledgeBaseStatus,
};
