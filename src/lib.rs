//! Bill-split demo for Coherence.
//!
//! The catalog in `.coherence/exports/catalog.jsonl` is the source of truth for
//! *what* this crate promises. The modules below are the implementation that the
//! acceptance criteria point at via codeintel links.

pub mod money;
pub mod split;
