//! Bounding, redaction, coordinate mapping, and the model-facing outline.

use super::{
    AccessibilityNode, AccessibilityScope, AccessibilitySnapshot, InputRect, StateFlags,
    Truncation, MAX_FIELD_CHARACTERS, MAX_OUTLINE_CHARACTERS,
};
use crate::DisplayGeometry;
use std::fmt::Write;

/// First line of every outline. Core uses it to recognize tree text that must
/// stay out of durable context and older provider turns.
pub const OUTLINE_MARKER: &str = "[computer accessibility tree]";

/// A rendered snapshot plus the redacted facts safe to persist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessibilityReport {
    pub text: String,
    pub scope: AccessibilityScope,
    pub node_count: usize,
    pub truncation: Option<Truncation>,
    pub inaccessible: bool,
    pub display_generation: u64,
}

/// Renders `snapshot` with boxes in the pixel space of `frame`, the latest
/// provider screenshot geometry.
pub fn render(
    snapshot: &AccessibilitySnapshot,
    scope: AccessibilityScope,
    frame: DisplayGeometry,
) -> AccessibilityReport {
    let nodes = prune(&snapshot.nodes);
    let mut truncation = snapshot.truncation;
    let mut body = String::new();
    let mut rendered = 0usize;
    for node in &nodes {
        let mut line = String::new();
        render_node(&mut line, rendered, node, frame);
        if body.len() + line.len() > MAX_OUTLINE_CHARACTERS {
            truncation = Some(Truncation::Size);
            break;
        }
        body.push_str(&line);
        rendered += 1;
    }
    let mut text = String::new();
    let _ = write!(
        text,
        "{OUTLINE_MARKER}\napp={} window={} generation={} nodes={rendered} truncated={}",
        quoted(snapshot.application.as_deref()),
        quoted(snapshot.window.as_deref()),
        frame.generation,
        truncation.map_or("none", Truncation::name),
    );
    if snapshot.inaccessible {
        text.push_str(" inaccessible=true");
    }
    text.push('\n');
    if rendered == 0 {
        text.push_str(if snapshot.inaccessible {
            "The target application refused accessibility inspection. Use screenshots instead.\n"
        } else {
            "No accessible elements were reported. Use screenshots instead.\n"
        });
    }
    text.push_str(&body);
    AccessibilityReport {
        text,
        scope,
        node_count: rendered,
        truncation,
        inaccessible: snapshot.inaccessible,
        display_generation: frame.generation,
    }
}

/// Removes unnamed structural wrappers, re-parenting their children.
fn prune(nodes: &[AccessibilityNode]) -> Vec<AccessibilityNode> {
    // Depth removed by skipped ancestors, indexed by original depth.
    let mut lift: Vec<u32> = Vec::new();
    let mut out = Vec::with_capacity(nodes.len());
    for (index, node) in nodes.iter().enumerate() {
        let depth = node.depth as usize;
        lift.truncate(depth);
        let inherited = lift.last().copied().unwrap_or(0);
        let children = nodes[index + 1..]
            .iter()
            .take_while(|next| next.depth > node.depth)
            .filter(|next| next.depth == node.depth + 1)
            .count();
        let skip = index > 0
            && node.role.is_structural()
            && node.name.is_none()
            && node.value.is_none()
            && node.states == StateFlags::empty()
            && children <= 1;
        lift.resize(depth, inherited);
        lift.push(inherited + u32::from(skip));
        if !skip {
            let mut kept = node.clone();
            kept.depth -= inherited;
            out.push(kept);
        }
    }
    out
}

fn render_node(line: &mut String, index: usize, node: &AccessibilityNode, frame: DisplayGeometry) {
    let _ = write!(
        line,
        "{:indent$}[{index}] {}",
        "",
        node.role.name(),
        indent = (node.depth as usize).min(40) * 2
    );
    if let Some(name) = node.name.as_deref().filter(|name| !name.is_empty()) {
        let _ = write!(line, " {}", quoted(Some(name)));
    }
    for state in node.states.names() {
        let _ = write!(line, " {state}");
    }
    if node.states.contains(StateFlags::PASSWORD) {
        line.push_str(" value=<redacted>");
    } else if let Some(value) = node.value.as_deref().filter(|value| !value.is_empty()) {
        let _ = write!(line, " value={}", quoted(Some(value)));
    }
    if let Some(description) = node
        .description
        .as_deref()
        .filter(|description| !description.is_empty() && Some(*description) != node.name.as_deref())
    {
        let _ = write!(line, " description={}", quoted(Some(description)));
    }
    if let Some([x, y, width, height]) = node.bounds.and_then(|bounds| pixel_box(bounds, frame)) {
        let _ = write!(line, " [{x},{y},{width},{height}]");
    }
    line.push('\n');
}

/// Bounded, single-line, quoted text. Longer text ends with `…(truncated)`.
fn quoted(value: Option<&str>) -> String {
    let Some(value) = value else {
        return "\"\"".into();
    };
    let mut out = String::from("\"");
    let mut truncated = false;
    for (index, character) in value.chars().enumerate() {
        if index == MAX_FIELD_CHARACTERS {
            truncated = true;
            break;
        }
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' | '\r' | '\t' => out.push(' '),
            character if character.is_control() => {}
            character => out.push(character),
        }
    }
    if truncated {
        out.push_str("…\"(truncated)");
    } else {
        out.push('"');
    }
    out
}

/// Maps an input-space rectangle into provider screenshot pixels, clipped to
/// the display. Returns `None` when nothing remains visible.
pub(crate) fn pixel_box(bounds: InputRect, frame: DisplayGeometry) -> Option<[u32; 4]> {
    if !(bounds.x.is_finite()
        && bounds.y.is_finite()
        && bounds.width.is_finite()
        && bounds.height.is_finite())
    {
        return None;
    }
    let left = to_pixel(
        bounds.x,
        frame.input_x,
        frame.input_width,
        frame.pixel_width,
    );
    let top = to_pixel(
        bounds.y,
        frame.input_y,
        frame.input_height,
        frame.pixel_height,
    );
    let right = to_pixel(
        bounds.x + bounds.width,
        frame.input_x,
        frame.input_width,
        frame.pixel_width,
    );
    let bottom = to_pixel(
        bounds.y + bounds.height,
        frame.input_y,
        frame.input_height,
        frame.pixel_height,
    );
    let (left, right) = (left.max(0.0), right.min(f64::from(frame.pixel_width)));
    let (top, bottom) = (top.max(0.0), bottom.min(f64::from(frame.pixel_height)));
    if right - left < 1.0 || bottom - top < 1.0 {
        return None;
    }
    let x = left.floor() as u32;
    let y = top.floor() as u32;
    Some([
        x,
        y,
        (right.ceil() as u32).saturating_sub(x).max(1),
        (bottom.ceil() as u32).saturating_sub(y).max(1),
    ])
}

/// Inverse of the frame's pixel-to-input axis mapping.
fn to_pixel(input: f64, origin: i32, input_extent: u32, pixel_extent: u32) -> f64 {
    if input_extent <= 1 || pixel_extent <= 1 {
        return 0.0;
    }
    (input - f64::from(origin)) * f64::from(pixel_extent - 1) / f64::from(input_extent - 1)
}

#[cfg(test)]
#[path = "outline_tests.rs"]
mod tests;
