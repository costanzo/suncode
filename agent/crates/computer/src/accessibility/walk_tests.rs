use super::*;

fn request(max_depth: u32, max_nodes: u32) -> AccessibilityRequest {
    AccessibilityRequest {
        target: AccessibilityTarget::FocusedWindow,
        max_depth,
        max_nodes,
        include_offscreen: false,
        display: InputRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
        deadline: Instant::now() + Duration::from_secs(5),
    }
}

/// A complete binary tree of `levels` levels with ids in heap order. Node 2
/// lies outside the display.
fn visit(levels: u32) -> impl FnMut(&u32) -> Option<Visited<u32>> {
    move |id| {
        let level = 31 - (id + 1).leading_zeros();
        Some(Visited {
            node: AccessibilityNode {
                depth: 0,
                role: Role::Button,
                name: Some(id.to_string()),
                value: None,
                description: None,
                states: StateFlags::empty(),
                bounds: (*id == 2).then_some(InputRect {
                    x: 500.0,
                    y: 0.0,
                    width: 10.0,
                    height: 10.0,
                }),
            },
            children: if level + 1 < levels {
                vec![id * 2 + 1, id * 2 + 2]
            } else {
                Vec::new()
            },
        })
    }
}

fn names(nodes: &[AccessibilityNode]) -> Vec<(String, u32)> {
    nodes
        .iter()
        .map(|node| (node.name.clone().unwrap(), node.depth))
        .collect()
}

#[test]
fn walk_is_pre_order_and_skips_offscreen_subtrees() {
    let request = request(10, 100);
    let never = || false;
    let mut budget = Budget::new(&request, &never);
    let mut nodes = Vec::new();
    walk(0u32, &request, &mut budget, &mut nodes, visit(3)).unwrap();
    assert_eq!(
        names(&nodes),
        [("0", 0), ("1", 1), ("3", 2), ("4", 2)].map(|(name, depth)| (name.to_string(), depth))
    );
    assert_eq!(budget.truncation(), None);
}

#[test]
fn offscreen_nodes_are_flagged_when_requested() {
    let mut request = request(10, 100);
    request.include_offscreen = true;
    let never = || false;
    let mut budget = Budget::new(&request, &never);
    let mut nodes = Vec::new();
    walk(0u32, &request, &mut budget, &mut nodes, visit(2)).unwrap();
    assert_eq!(nodes.len(), 3);
    assert!(nodes[2].states.contains(StateFlags::OFFSCREEN));
    assert!(!nodes[1].states.contains(StateFlags::OFFSCREEN));
}

#[test]
fn walk_records_depth_and_node_limits() {
    let never = || false;
    let shallow = request(1, 100);
    let mut budget = Budget::new(&shallow, &never);
    let mut nodes = Vec::new();
    walk(0u32, &shallow, &mut budget, &mut nodes, visit(4)).unwrap();
    assert_eq!(nodes.len(), 2);
    assert_eq!(budget.truncation(), Some(Truncation::Depth));

    let small = request(10, 3);
    let mut budget = Budget::new(&small, &never);
    let mut nodes = Vec::new();
    walk(0u32, &small, &mut budget, &mut nodes, visit(4)).unwrap();
    assert_eq!(nodes.len(), 3);
    assert_eq!(budget.truncation(), Some(Truncation::Nodes));
}

#[test]
fn walk_stops_at_the_deadline_with_partial_results() {
    let mut request = request(10, 100);
    request.deadline = Instant::now();
    let never = || false;
    let mut budget = Budget::new(&request, &never);
    let mut nodes = Vec::new();
    walk(0u32, &request, &mut budget, &mut nodes, visit(3)).unwrap();
    assert_eq!(names(&nodes), vec![("0".to_string(), 0)]);
    assert_eq!(budget.truncation(), Some(Truncation::Deadline));
}

#[test]
fn walk_stops_on_cancellation() {
    let request = request(10, 100);
    let cancelled = || true;
    let mut budget = Budget::new(&request, &cancelled);
    let mut nodes = Vec::new();
    assert_eq!(
        walk(0u32, &request, &mut budget, &mut nodes, visit(3)),
        Err(crate::ComputerError::Cancelled)
    );
}

#[test]
fn clean_trims_and_bounds_text() {
    assert_eq!(clean("  "), None);
    assert_eq!(clean(" a ").as_deref(), Some("a"));
    assert_eq!(
        clean(&"x".repeat(1_000)).unwrap().chars().count(),
        MAX_FIELD_CHARACTERS + 1
    );
}
