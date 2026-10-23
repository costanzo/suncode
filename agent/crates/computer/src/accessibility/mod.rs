//! Read-only accessibility snapshots of the primary display.
//!
//! Platform collectors report nodes in input (logical) coordinates. This
//! module bounds and redacts them, maps their boxes into the latest provider
//! screenshot, and renders the outline the model receives. Nothing here
//! mutates the target application.

mod outline;
mod vocabulary;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub use outline::{render, AccessibilityReport, OUTLINE_MARKER};
pub use vocabulary::{Role, StateFlags};

use crate::{ComputerError, ComputerResult, PermissionState, PixelPoint};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

pub const DEFAULT_MAX_DEPTH: u32 = 12;
pub const MAX_DEPTH: u32 = 25;
pub const DEFAULT_MAX_NODES: u32 = 300;
pub const MAX_NODES: u32 = 1_000;
/// Characters kept from one name, value, or description.
pub const MAX_FIELD_CHARACTERS: usize = 200;
/// Characters kept from the rendered outline.
pub const MAX_OUTLINE_CHARACTERS: usize = 120_000;
pub const COLLECTION_DEADLINE: Duration = Duration::from_secs(2);

/// Which part of the desktop to describe.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AccessibilityScope {
    #[default]
    FocusedWindow,
    FrontmostApplication,
    Point,
}

impl AccessibilityScope {
    pub const fn name(self) -> &'static str {
        match self {
            Self::FocusedWindow => "focused_window",
            Self::FrontmostApplication => "frontmost_application",
            Self::Point => "point",
        }
    }
}

/// Model-facing request in screenshot pixels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AccessibilityQuery {
    #[serde(default)]
    pub scope: AccessibilityScope,
    #[serde(default)]
    pub coordinate: Option<PixelPoint>,
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,
    #[serde(default = "default_max_nodes")]
    pub max_nodes: u32,
    #[serde(default)]
    pub include_offscreen: bool,
}

impl AccessibilityQuery {
    /// Parses and validates `computer_accessibility_tree` arguments.
    pub fn from_input(input: &serde_json::Value) -> ComputerResult<Self> {
        let mut object = input
            .as_object()
            .cloned()
            .ok_or(ComputerError::InvalidAction(
                "computer member input must be an object",
            ))?;
        if let Some(value) = object.get_mut("coordinate") {
            if !value.is_null() {
                *value = crate::action::normalize_point(value)?;
            }
        }
        let query: Self = serde_json::from_value(serde_json::Value::Object(object))
            .map_err(|_| ComputerError::InvalidAction("accessibility tree input is invalid"))?;
        query.validate()?;
        Ok(query)
    }

    pub fn validate(&self) -> ComputerResult<()> {
        if !(1..=MAX_DEPTH).contains(&self.max_depth) {
            return Err(ComputerError::InvalidAction(
                "max_depth must be between 1 and 25",
            ));
        }
        if !(1..=MAX_NODES).contains(&self.max_nodes) {
            return Err(ComputerError::InvalidAction(
                "max_nodes must be between 1 and 1000",
            ));
        }
        match (self.scope, self.coordinate) {
            (AccessibilityScope::Point, None) => Err(ComputerError::InvalidAction(
                "point scope requires a coordinate",
            )),
            (AccessibilityScope::Point, Some(_)) | (_, None) => Ok(()),
            (_, Some(_)) => Err(ComputerError::InvalidAction(
                "coordinate is only valid with point scope",
            )),
        }
    }
}

const fn default_max_depth() -> u32 {
    DEFAULT_MAX_DEPTH
}

const fn default_max_nodes() -> u32 {
    DEFAULT_MAX_NODES
}

/// Backend request in input coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct AccessibilityRequest {
    pub target: AccessibilityTarget,
    pub max_depth: u32,
    pub max_nodes: u32,
    pub include_offscreen: bool,
    /// Visible input-space rectangle of the primary display.
    pub display: InputRect,
    pub deadline: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessibilityTarget {
    FocusedWindow,
    FrontmostApplication,
    Point { x: i32, y: i32 },
}

/// A rectangle in input (logical) coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl InputRect {
    pub fn intersects(&self, other: &Self) -> bool {
        self.width > 0.0
            && self.height > 0.0
            && other.width > 0.0
            && other.height > 0.0
            && self.x < other.x + other.width
            && other.x < self.x + self.width
            && self.y < other.y + other.height
            && other.y < self.y + self.height
    }
}

/// One element as reported by a platform collector.
///
/// Collectors leave `value` empty and set [`StateFlags::PASSWORD`] for secure
/// text, so protected values never leave the platform module.
#[derive(Debug, Clone, PartialEq)]
pub struct AccessibilityNode {
    /// Depth below the snapshot root (the root is 0).
    pub depth: u32,
    pub role: Role,
    pub name: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
    pub states: StateFlags,
    pub bounds: Option<InputRect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Truncation {
    Nodes,
    Depth,
    Deadline,
    /// The rendered outline reached its character bound.
    Size,
}

impl Truncation {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Nodes => "nodes",
            Self::Depth => "depth",
            Self::Deadline => "deadline",
            Self::Size => "size",
        }
    }
}

/// Depth-first, depth-annotated nodes from one collection.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AccessibilitySnapshot {
    pub application: Option<String>,
    pub window: Option<String>,
    pub nodes: Vec<AccessibilityNode>,
    pub truncation: Option<Truncation>,
    /// The target exists but refused inspection (elevated or secure input).
    pub inaccessible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessibilityRuntimeInfo {
    pub available: bool,
    pub permission: PermissionState,
}

/// Read-only, OS-facing accessibility snapshot source.
///
/// Implementations are called from a collector thread without the executor
/// lock held, so they must tolerate running beside input or stop requests.
pub trait AccessibilityBackend: Send + Sync {
    fn runtime_info(&self) -> AccessibilityRuntimeInfo;
    fn snapshot(
        &self,
        request: &AccessibilityRequest,
        cancelled: &dyn Fn() -> bool,
    ) -> ComputerResult<AccessibilitySnapshot>;
}

/// The built-in backend for this platform, if one exists. Construction may
/// block briefly while it connects to platform services.
pub fn platform_backend() -> Option<Box<dyn AccessibilityBackend>> {
    #[cfg(target_os = "macos")]
    return Some(Box::new(macos::MacAccessibility::new()));
    #[cfg(target_os = "windows")]
    return Some(Box::new(windows::UiaAccessibility::new()));
    #[cfg(target_os = "linux")]
    return linux::AtspiAccessibility::new()
        .map(|backend| Box::new(backend) as Box<dyn AccessibilityBackend>);
    #[allow(unreachable_code)]
    None
}

/// Accessibility permission without building a backend or prompting.
pub fn platform_permission() -> PermissionState {
    #[cfg(target_os = "macos")]
    return macos::permission();
    #[cfg(target_os = "windows")]
    return PermissionState::Allowed;
    #[cfg(target_os = "linux")]
    return linux::permission();
    #[allow(unreachable_code)]
    PermissionState::Unsupported
}

/// Shared walk limits for platform collectors.
pub(crate) struct Budget<'a> {
    request: &'a AccessibilityRequest,
    cancelled: &'a dyn Fn() -> bool,
    stopped: Option<Truncation>,
    depth_limited: bool,
}

impl<'a> Budget<'a> {
    pub(crate) fn new(request: &'a AccessibilityRequest, cancelled: &'a dyn Fn() -> bool) -> Self {
        Self {
            request,
            cancelled,
            stopped: None,
            depth_limited: false,
        }
    }

    /// Whether the walk must stop; records why when a limit was reached.
    pub(crate) fn exhausted(&mut self, collected: usize) -> ComputerResult<bool> {
        if (self.cancelled)() {
            return Err(crate::ComputerError::Cancelled);
        }
        if self.stopped.is_none() {
            if collected >= self.request.max_nodes as usize {
                self.stopped = Some(Truncation::Nodes);
            } else if collected > 0 && Instant::now() >= self.request.deadline {
                // The root is always described so a slow start still yields
                // a usable header node.
                self.stopped = Some(Truncation::Deadline);
            }
        }
        Ok(self.stopped.is_some())
    }

    /// Whether children of a node at `depth` may be visited. Skipped
    /// subtrees are recorded but do not stop the walk.
    pub(crate) fn descend(&mut self, depth: u32) -> bool {
        if depth >= self.request.max_depth {
            self.depth_limited = true;
            return false;
        }
        true
    }

    pub(crate) fn remaining(&self) -> Duration {
        self.request
            .deadline
            .saturating_duration_since(Instant::now())
    }

    pub(crate) fn truncation(&self) -> Option<Truncation> {
        self.stopped
            .or(self.depth_limited.then_some(Truncation::Depth))
    }
}

/// One described element and the children it reported.
pub(crate) struct Visited<E> {
    pub(crate) node: AccessibilityNode,
    pub(crate) children: Vec<E>,
}

/// Depth-first pre-order walk from `root`, appending to `nodes`.
///
/// `visit` returns `None` for elements that disappeared or refused
/// inspection; they are skipped with their subtree. Offscreen elements are
/// skipped with their subtree unless the request includes them.
pub(crate) fn walk<E>(
    root: E,
    request: &AccessibilityRequest,
    budget: &mut Budget<'_>,
    nodes: &mut Vec<AccessibilityNode>,
    mut visit: impl FnMut(&E) -> Option<Visited<E>>,
) -> ComputerResult<()> {
    let mut stack = vec![(root, 0u32)];
    while let Some((element, depth)) = stack.pop() {
        if budget.exhausted(nodes.len())? {
            break;
        }
        let Some(Visited { mut node, children }) = visit(&element) else {
            continue;
        };
        if !node
            .bounds
            .is_none_or(|bounds| bounds.intersects(&request.display))
        {
            if !request.include_offscreen {
                continue;
            }
            node.states.insert(StateFlags::OFFSCREEN);
        }
        node.depth = depth;
        nodes.push(node);
        if !children.is_empty() && budget.descend(depth) {
            stack.extend(children.into_iter().rev().map(|child| (child, depth + 1)));
        }
    }
    Ok(())
}

/// Bounded single-line text from a platform string; empty becomes `None`.
pub(crate) fn clean(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Keep one extra character so rendering can mark truncation.
    Some(trimmed.chars().take(MAX_FIELD_CHARACTERS + 1).collect())
}

#[cfg(test)]
#[path = "walk_tests.rs"]
mod tests;
