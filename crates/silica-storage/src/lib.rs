//! Storage and persistence boundary for SilicaRAW.
//!
//! Spike 004 selects rusqlite with bundled SQLite and embedded SQL migrations.
//! This crate owns catalog schema creation, library-local sidecars, cache
//! records, and dry-run recovery reports. It does not decode photos, mutate
//! originals, write next-to-original sidecars, or apply restore actions yet.

mod actions;
mod backup;
mod cache;
mod common;
mod edits;
mod exports;
mod library;
mod migrations;
mod model;
mod photos;
mod sidecar;

pub use silica_catalog::{ImportIssue, ImportIssueKind};
pub use silica_catalog::{
    LibraryQueryFileType, LibraryQueryFilters, LibraryQueryMetadataFilter, LibraryQueryOrderField,
    LibraryQueryPage, LibraryQueryRequest, LibraryQuerySort, PhotoFlags,
};

pub use actions::*;
pub use backup::*;
pub use cache::*;
pub use edits::*;
pub use exports::*;
pub use library::*;
pub use migrations::*;
pub use model::*;
pub use photos::*;
pub use sidecar::*;

#[cfg(test)]
mod tests;
