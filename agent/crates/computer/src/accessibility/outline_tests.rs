use super::*;
use crate::Role;

fn frame(input: (u32, u32), pixels: (u32, u32)) -> DisplayGeometry {
    DisplayGeometry {
        generation: 3,
        input_x: 0,
        input_y: 0,
        input_width: input.0,
        input_height: input.1,
        pixel_width: pixels.0,
        pixel_height: pixels.1,
    }
}

fn node(depth: u32, role: Role, name: Option<&str>) -> AccessibilityNode {
    AccessibilityNode {
        depth,
        role,
        name: name.map(Into::into),
        value: None,
        description: None,
        states: StateFlags::empty(),
        bounds: None,
    }
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> InputRect {
    InputRect {
        x,
        y,
        width,
        height,
    }
}

#[test]
fn boxes_follow_the_screenshot_scale() {
    // Provider image is half the input resolution.
    let geometry = frame((2_001, 1_001), (1_001, 501));
    assert_eq!(
        pixel_box(rect(200.0, 100.0, 400.0, 200.0), geometry),
        Some([100, 50, 200, 100])
    );
    // Identity mapping keeps boxes unchanged.
    let identity = frame((1_440, 900), (1_440, 900));
    assert_eq!(
        pixel_box(rect(10.0, 20.0, 30.0, 40.0), identity),
        Some([10, 20, 30, 40])
    );
}

#[test]
fn boxes_are_clipped_or_dropped_outside_the_display() {
    let geometry = frame((1_000, 1_000), (1_000, 1_000));
    assert_eq!(
        pixel_box(rect(-50.0, 990.0, 100.0, 100.0), geometry),
        Some([0, 990, 50, 10])
    );
    assert_eq!(pixel_box(rect(2_000.0, 0.0, 10.0, 10.0), geometry), None);
    assert_eq!(pixel_box(rect(f64::NAN, 0.0, 10.0, 10.0), geometry), None);
}

#[test]
fn box_centers_round_trip_through_input_mapping() {
    let geometry = frame((1_512, 982), (1_456, 945));
    let bounds = rect(700.0, 400.0, 80.0, 24.0);
    let [x, y, width, height] = pixel_box(bounds, geometry).unwrap();
    let source = crate::ComputerFrame::new(
        geometry,
        vec![0; geometry.pixel_width as usize * geometry.pixel_height as usize * 4],
    )
    .unwrap();
    let (input_x, input_y) = source
        .input_point(crate::PixelPoint {
            x: x + width / 2,
            y: y + height / 2,
        })
        .unwrap();
    assert!((f64::from(input_x) - 740.0).abs() <= 2.0, "{input_x}");
    assert!((f64::from(input_y) - 412.0).abs() <= 2.0, "{input_y}");
}

#[test]
fn unnamed_single_child_wrappers_are_pruned() {
    let nodes = vec![
        node(0, Role::Window, Some("Main")),
        node(1, Role::Group, None),
        node(2, Role::Group, None),
        node(3, Role::Button, Some("OK")),
        node(1, Role::Group, None),
        node(2, Role::Text, Some("a")),
        node(2, Role::Text, Some("b")),
    ];
    let pruned = prune(&nodes);
    let shape: Vec<_> = pruned
        .iter()
        .map(|node| (node.depth, node.role, node.name.as_deref()))
        .collect();
    assert_eq!(
        shape,
        vec![
            (0, Role::Window, Some("Main")),
            (1, Role::Button, Some("OK")),
            (1, Role::Group, None),
            (2, Role::Text, Some("a")),
            (2, Role::Text, Some("b")),
        ]
    );
}

#[test]
fn passwords_are_redacted_and_fields_are_bounded() {
    let mut password = node(1, Role::TextField, Some("Password"));
    password.states.insert(StateFlags::PASSWORD);
    // A collector bug must still not leak the value.
    password.value = Some("hunter2".into());
    let mut long = node(1, Role::Text, Some("Body"));
    long.value = Some("x".repeat(MAX_FIELD_CHARACTERS + 50));
    let snapshot = AccessibilitySnapshot {
        application: Some("App \"One\"".into()),
        window: Some("Line\nbreak".into()),
        nodes: vec![node(0, Role::Window, Some("w")), password, long],
        ..AccessibilitySnapshot::default()
    };
    let report = render(
        &snapshot,
        AccessibilityScope::FocusedWindow,
        frame((10, 10), (10, 10)),
    );
    assert!(report.text.starts_with(OUTLINE_MARKER));
    assert!(!report.text.contains("hunter2"));
    assert!(report.text.contains("password value=<redacted>"));
    assert!(report.text.contains("…\"(truncated)"));
    assert!(report
        .text
        .contains("app=\"App \\\"One\\\"\" window=\"Line break\""));
    assert_eq!(report.node_count, 3);
    assert_eq!(report.display_generation, 3);
}

#[test]
fn outline_size_is_bounded() {
    let nodes = (0..5_000)
        .map(|index| {
            let mut node = node(1, Role::Text, Some(&"n".repeat(150)));
            node.value = Some(format!("{index}{}", "v".repeat(150)));
            node
        })
        .collect();
    let snapshot = AccessibilitySnapshot {
        nodes,
        ..AccessibilitySnapshot::default()
    };
    let report = render(
        &snapshot,
        AccessibilityScope::FrontmostApplication,
        frame((10, 10), (10, 10)),
    );
    assert!(report.text.len() <= MAX_OUTLINE_CHARACTERS + 512);
    assert_eq!(report.truncation, Some(Truncation::Size));
    assert!(report.text.contains("truncated=size"));
}

#[test]
fn empty_and_inaccessible_trees_explain_themselves() {
    let geometry = frame((10, 10), (10, 10));
    let empty = render(
        &AccessibilitySnapshot::default(),
        AccessibilityScope::FocusedWindow,
        geometry,
    );
    assert_eq!(empty.node_count, 0);
    assert!(empty.text.contains("No accessible elements"));
    let refused = render(
        &AccessibilitySnapshot {
            inaccessible: true,
            ..AccessibilitySnapshot::default()
        },
        AccessibilityScope::FocusedWindow,
        geometry,
    );
    assert!(refused.inaccessible && refused.text.contains("inaccessible=true"));
}
