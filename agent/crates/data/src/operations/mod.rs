//! Table-owned Diesel operations. Cross-table event projection is kept in `projection`.

pub(crate) mod approval_request;
pub(crate) mod checkpoint;
pub(crate) mod checkpoint_manifest;
pub(crate) mod configuration;
pub(crate) mod llm_model;
pub(crate) mod llm_model_provider;
pub(crate) mod project;
pub(crate) mod project_dependency;
pub(crate) mod session;
pub(crate) mod session_call;
pub(crate) mod session_image;
pub(crate) mod session_message;
pub(crate) mod session_tool_use;
pub(crate) mod session_turn;
pub(crate) mod session_turn_todo;

pub(crate) mod projection;
pub(crate) mod recovery;
