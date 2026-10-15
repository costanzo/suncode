//! Local, file-backed Agent Skills discovery and loading.
//!
//! This crate owns all Skill filesystem access. The agent core only consumes
//! validated metadata and bounded documents returned by this API.
//!
//! - `catalog`: discovery precedence, project boundaries, collisions, lookup.
//! - `manifest`: `SKILL.md` frontmatter parsing and validation.
//! - `scan`: filesystem traversal for skill files and resources.
//! - `render`: the model-facing `<available_skills>` listing.

mod catalog;
mod diagnostic;
mod manifest;
mod model;
mod render;
mod scan;

pub use catalog::{DiscoveryOptions, SkillCatalog, MAX_RESOURCE_FILES};
pub use diagnostic::{DiagnosticKind, SkillDiagnostic};
pub use manifest::{MAX_DESCRIPTION_CHARS, MAX_NAME_CHARS, MAX_SKILL_BYTES};
pub use model::{SkillDocument, SkillInfo, SkillSource, SKILL_FILE_NAME};
pub use render::render_available_skills;
