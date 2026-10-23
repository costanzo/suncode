//! [`ComputerRuntime`] as a tool provider for the `computer_*` tools.
//!
//! Models with native computer use emit `toolset_name = "computer"` calls with
//! a bare member name; the core canonicalizes those to `computer_<member>`
//! before routing, so this provider only sees prefixed names.

use crate::runtime::{computer_tool_definitions, is_known_member, ComputerRuntime};
use crate::{AccessibilityQuery, ActionOutcome, ComputerAction, Truncation};
use serde_json::{json, Value};
use suncode_common::tool::{
    ApprovalTarget, BoxFuture, CatalogContext, Risk, ToolContext, ToolDefinition, ToolImage,
    ToolInvocation, ToolProvider, ToolResult,
};
use suncode_common::BusinessError;

const PREFIX: &str = "computer_";
const ACCESSIBILITY_TOOL: &str = "computer_accessibility_tree";
/// Approval and failure-halt batch shared by consecutive computer calls.
const BATCH: &str = "computer";

/// The executor member for a `computer_<member>` tool name.
pub fn member(name: &str) -> Option<&str> {
    name.strip_prefix(PREFIX)
        .filter(|member| is_known_member(member))
}

impl ToolProvider for ComputerRuntime {
    fn owns(&self, name: &str) -> bool {
        member(name).is_some()
    }

    fn catalog<'a>(
        &'a self,
        context: &'a CatalogContext<'a>,
    ) -> BoxFuture<'a, Vec<ToolDefinition>> {
        Box::pin(async move {
            if !self.advertises_tools(context.computer_use) {
                return Vec::new();
            }
            let accessibility = self.advertises_accessibility();
            computer_tool_definitions()
                .into_iter()
                .filter(|tool| accessibility || tool.name != ACCESSIBILITY_TOOL)
                .collect()
        })
    }

    fn validate(&self, name: &str, arguments: &Value) -> Result<(), BusinessError> {
        let member = member(name).ok_or_else(|| BusinessError::invalid("unknown computer tool"))?;
        if member == "accessibility_tree" {
            return AccessibilityQuery::from_input(arguments)
                .map(|_| ())
                .map_err(|error| BusinessError::invalid(error.to_string()));
        }
        ComputerAction::from_member(member, arguments)
            .map(|_| ())
            .map_err(|error| BusinessError::invalid(error.to_string()))
    }

    fn risk(&self, name: &str) -> Option<Risk> {
        match member(name)? {
            "screenshot" | "zoom" | "cursor_position" | "wait" | "mouse_move"
            | "accessibility_tree" => Some(Risk::ComputerObserve),
            "left_click" | "right_click" | "middle_click" | "double_click" | "triple_click"
            | "left_click_drag" | "left_mouse_down" | "left_mouse_up" | "scroll" | "type"
            | "key" | "hold_key" => Some(Risk::ComputerInput),
            _ => None,
        }
    }

    fn approval_target<'a>(
        &'a self,
        _project_id: &'a str,
        name: &'a str,
        arguments: &'a Value,
    ) -> BoxFuture<'a, Result<ApprovalTarget, BusinessError>> {
        Box::pin(async move {
            let member = member(name).unwrap_or(name);
            Ok(ApprovalTarget {
                operation: format!("computer.{member}"),
                arguments: json!({"toolsetName":"computer","member":member,"input":arguments}),
                pin: None,
            })
        })
    }

    fn approval_batch(&self, _name: &str) -> Option<&'static str> {
        Some(BATCH)
    }

    /// Actions drive one shared display and input device.
    fn concurrent(&self, _name: &str) -> bool {
        false
    }

    fn execute<'a>(
        &'a self,
        invocation: ToolInvocation<'a>,
        context: ToolContext<'a>,
    ) -> BoxFuture<'a, Result<ToolResult, BusinessError>> {
        Box::pin(async move {
            let member = member(invocation.name)
                .ok_or_else(|| BusinessError::invalid("unknown computer tool"))?;
            let outcome = self
                .run_action(member, &invocation.arguments, context.cancellation)
                .await?;
            self.tool_result(outcome)
        })
    }
}

impl ComputerRuntime {
    /// Text results pass through; screenshots are stored as artifacts and
    /// attached to the model message as an image.
    fn tool_result(&self, outcome: ActionOutcome) -> Result<ToolResult, BusinessError> {
        match outcome {
            ActionOutcome::Accessibility { report, duration } => Ok(ToolResult {
                // Only redacted facts reach history, events, and SQLite.
                value: json!({
                    "type":"computer_accessibility",
                    "scope":report.scope.name(),
                    "nodeCount":report.node_count,
                    "truncated":report.truncation.is_some(),
                    "truncationReason":report.truncation.map(Truncation::name),
                    "inaccessible":report.inaccessible,
                    "displayGeneration":report.display_generation,
                    "durationMs":u64::try_from(duration.as_millis()).unwrap_or(u64::MAX),
                }),
                model_text: Some(report.text),
                image: None,
            }),
            ActionOutcome::Text(text) => Ok(ToolResult {
                value: json!({"type":"computer_text","text":text}),
                model_text: Some(text),
                image: None,
            }),
            ActionOutcome::Image {
                frame,
                source_frame,
            } => {
                let capture_failed = |error: crate::ComputerError| {
                    BusinessError::new("computer_capture_failed", error.to_string())
                };
                let source_png = source_frame.png().map_err(capture_failed)?;
                let provider_png = frame.png().map_err(capture_failed)?;
                let artifact_id = self.persist_screenshot(&source_png)?;
                Ok(ToolResult {
                    value: json!({
                        "type":"computer_image",
                        "artifact_id":artifact_id,
                        "relative_path":format!("computer/artifacts/{artifact_id}.png"),
                        "mime_type":"image/png",
                        "bytes":source_png.len(),
                        "pixelWidth":frame.display.pixel_width,
                        "pixelHeight":frame.display.pixel_height,
                        "sourcePixelWidth":source_frame.display.pixel_width,
                        "sourcePixelHeight":source_frame.display.pixel_height,
                        "displayGeneration":frame.display.generation,
                    }),
                    model_text: Some(format!(
                        "Computer screenshot captured and saved as artifact {artifact_id} at computer/artifacts/{artifact_id}.png."
                    )),
                    image: Some(ToolImage {
                        mime_type: "image/png".into(),
                        bytes: provider_png,
                    }),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ComputerBackend, ComputerFrame, ComputerResult, DisplayGeometry, KeyState, MouseButton,
        MouseButtonState, ScrollDirection,
    };
    use std::sync::Arc;
    use suncode_common::tool::{
        BoxFuture, Cancellation, Document, DocumentReader, PathScopes, ScopedPath, ToolOutput,
    };

    struct Screen;
    impl ComputerBackend for Screen {
        fn primary_display(&mut self) -> ComputerResult<DisplayGeometry> {
            Ok(DisplayGeometry {
                generation: 1,
                input_x: 0,
                input_y: 0,
                input_width: 4,
                input_height: 2,
                pixel_width: 8,
                pixel_height: 4,
            })
        }
        fn capture_primary(&mut self) -> ComputerResult<ComputerFrame> {
            ComputerFrame::new(self.primary_display()?, vec![0; 8 * 4 * 4])
        }
        fn move_pointer(&mut self, _: i32, _: i32) -> ComputerResult<()> {
            Ok(())
        }
        fn pointer_position(&mut self) -> ComputerResult<(i32, i32)> {
            Ok((1, 1))
        }
        fn mouse_button(&mut self, _: MouseButton, _: MouseButtonState) -> ComputerResult<()> {
            Ok(())
        }
        fn scroll(&mut self, _: ScrollDirection, _: u32) -> ComputerResult<()> {
            Ok(())
        }
        fn type_text(&mut self, _: &str) -> ComputerResult<()> {
            Ok(())
        }
        fn key(&mut self, _: &str, _: KeyState) -> ComputerResult<()> {
            Ok(())
        }
    }

    struct Paths;
    impl PathScopes for Paths {
        fn resolve(&self, path: &str) -> Result<ScopedPath, BusinessError> {
            Ok(ScopedPath::Project {
                relative: path.into(),
            })
        }
        fn display_path(&self, _: &std::path::Path) -> Option<String> {
            None
        }
    }
    struct NoDocuments;
    impl DocumentReader for NoDocuments {
        fn read_text<'a>(
            &'a self,
            _: &'a str,
            _: u64,
        ) -> BoxFuture<'a, Result<Document, BusinessError>> {
            Box::pin(async { Err(BusinessError::invalid("no documents")) })
        }
    }
    struct Sink;
    impl ToolOutput for Sink {
        fn chunk(&self, _: &str, _: &[u8]) {}
    }
    struct Never;
    impl Cancellation for Never {
        fn is_cancelled(&self) -> bool {
            false
        }
        fn cancelled(&self) -> BoxFuture<'_, ()> {
            Box::pin(std::future::pending())
        }
    }

    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        struct Unpark(std::thread::Thread);
        impl std::task::Wake for Unpark {
            fn wake(self: Arc<Self>) {
                self.0.unpark();
            }
        }
        let waker = std::task::Waker::from(Arc::new(Unpark(std::thread::current())));
        let mut context = std::task::Context::from_waker(&waker);
        let mut future = std::pin::pin!(future);
        loop {
            if let std::task::Poll::Ready(value) = future.as_mut().poll(&mut context) {
                return value;
            }
            std::thread::park();
        }
    }

    fn runtime(data: &std::path::Path) -> ComputerRuntime {
        let runtime = ComputerRuntime::new(true, true, data.to_path_buf());
        runtime.install_backend(Box::new(Screen)).unwrap();
        runtime
    }

    #[test]
    fn names_risk_and_batch_follow_the_computer_contract() {
        let data = tempfile::tempdir().unwrap();
        let runtime = runtime(data.path());
        assert!(runtime.owns("computer_left_click") && !runtime.owns("left_click"));
        assert!(!runtime.owns("computer_unknown"));
        assert_eq!(
            runtime.risk("computer_screenshot"),
            Some(Risk::ComputerObserve)
        );
        assert_eq!(runtime.risk("computer_type"), Some(Risk::ComputerInput));
        assert_eq!(runtime.approval_batch("computer_type"), Some("computer"));
        assert!(!runtime.concurrent("computer_screenshot"));
        let target =
            block_on(runtime.approval_target("p", "computer_type", &json!({"text":"a"}))).unwrap();
        assert_eq!(target.operation, "computer.type");
        assert_eq!(target.arguments["member"], "type");
    }

    #[test]
    fn catalog_requires_a_model_with_computer_use() {
        let data = tempfile::tempdir().unwrap();
        let runtime = runtime(data.path());
        let unsupported = CatalogContext {
            project_id: "p",
            computer_use: false,
        };
        assert!(block_on(runtime.catalog(&unsupported)).is_empty());
        let supported = CatalogContext {
            project_id: "p",
            computer_use: true,
        };
        let tools = block_on(runtime.catalog(&supported));
        assert_eq!(tools.len(), 17);
        for tool in tools {
            assert!(
                jsonschema::validator_for(&tool.parameters).is_ok(),
                "{}",
                tool.name
            );
        }
        let hidden = ComputerRuntime::new(true, false, data.path().to_path_buf());
        assert!(block_on(hidden.catalog(&supported)).is_empty());
    }

    #[test]
    fn screenshots_are_stored_and_attached_as_images() {
        let data = tempfile::tempdir().unwrap();
        let runtime = runtime(data.path());
        let project = tempfile::tempdir().unwrap();
        let result = block_on(runtime.execute(
            ToolInvocation {
                name: "computer_screenshot",
                arguments: json!({}),
                approval_pin: None,
            },
            ToolContext {
                project_id: "p",
                project_root: project.path(),
                call_key: "turn:call",
                timeout_ceiling: None,
                paths: &Paths,
                documents: &NoDocuments,
                output: Arc::new(Sink),
                cancellation: &Never,
            },
        ))
        .unwrap();
        assert_eq!(result.value["type"], "computer_image");
        let artifact = result.value["artifact_id"].as_str().unwrap();
        let stored = data
            .path()
            .join("computer/artifacts")
            .join(format!("{artifact}.png"));
        assert!(stored.is_file());
        assert!(result
            .model_text
            .unwrap()
            .starts_with("Computer screenshot captured"));
        let image = result.image.unwrap();
        assert_eq!(image.mime_type, "image/png");
        assert!(image.bytes.starts_with(b"\x89PNG"));
    }

    struct FakeTree {
        delay: std::time::Duration,
        password: &'static str,
    }

    impl crate::AccessibilityBackend for FakeTree {
        fn runtime_info(&self) -> crate::AccessibilityRuntimeInfo {
            crate::AccessibilityRuntimeInfo {
                available: true,
                permission: crate::PermissionState::Allowed,
            }
        }

        fn snapshot(
            &self,
            request: &crate::AccessibilityRequest,
            cancelled: &dyn Fn() -> bool,
        ) -> ComputerResult<crate::AccessibilitySnapshot> {
            let until = std::time::Instant::now() + self.delay;
            while std::time::Instant::now() < until {
                if cancelled() {
                    return Err(crate::ComputerError::Cancelled);
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            assert_eq!(request.target, crate::AccessibilityTarget::FocusedWindow);
            let mut password = crate::AccessibilityNode {
                depth: 1,
                role: crate::Role::TextField,
                name: Some("Password".into()),
                value: Some(self.password.into()),
                description: None,
                states: crate::StateFlags::PASSWORD,
                bounds: Some(crate::InputRect {
                    x: 1.0,
                    y: 0.5,
                    width: 2.0,
                    height: 1.0,
                }),
            };
            password.states.insert(crate::StateFlags::FOCUSED);
            Ok(crate::AccessibilitySnapshot {
                application: Some("Fixture".into()),
                window: Some("Login".into()),
                nodes: vec![
                    crate::AccessibilityNode {
                        depth: 0,
                        role: crate::Role::Window,
                        name: Some("Login".into()),
                        value: None,
                        description: None,
                        states: crate::StateFlags::empty(),
                        bounds: None,
                    },
                    password,
                ],
                truncation: None,
                inaccessible: false,
            })
        }
    }

    fn tree_runtime(data: &std::path::Path, delay_ms: u64) -> ComputerRuntime {
        let runtime = runtime(data);
        runtime
            .install_accessibility_backend(Box::new(FakeTree {
                delay: std::time::Duration::from_millis(delay_ms),
                password: "hunter2",
            }))
            .unwrap();
        runtime
    }

    fn call(
        runtime: &ComputerRuntime,
        name: &'static str,
        arguments: Value,
    ) -> Result<ToolResult, BusinessError> {
        let project = tempfile::tempdir().unwrap();
        block_on(runtime.execute(
            ToolInvocation {
                name,
                arguments,
                approval_pin: None,
            },
            ToolContext {
                project_id: "p",
                project_root: project.path(),
                call_key: "turn:call",
                timeout_ceiling: None,
                paths: &Paths,
                documents: &NoDocuments,
                output: Arc::new(Sink),
                cancellation: &Never,
            },
        ))
    }

    #[test]
    fn accessibility_tool_needs_a_backend_and_observe_risk() {
        let data = tempfile::tempdir().unwrap();
        let supported = CatalogContext {
            project_id: "p",
            computer_use: true,
        };
        let plain = runtime(data.path());
        assert!(!block_on(plain.catalog(&supported))
            .iter()
            .any(|tool| tool.name == ACCESSIBILITY_TOOL));
        let tree = tree_runtime(data.path(), 0);
        assert_eq!(block_on(tree.catalog(&supported)).len(), 18);
        assert_eq!(tree.risk(ACCESSIBILITY_TOOL), Some(Risk::ComputerObserve));
        assert!(tree
            .validate(ACCESSIBILITY_TOOL, &json!({"scope":"point"}))
            .is_err());
        assert!(tree
            .validate(ACCESSIBILITY_TOOL, &json!({"coordinate":[1,1]}))
            .is_err());
        assert!(tree
            .validate(ACCESSIBILITY_TOOL, &json!({"max_nodes":0}))
            .is_err());
        assert!(tree
            .validate(
                ACCESSIBILITY_TOOL,
                &json!({"scope":"point","coordinate":[1,1]})
            )
            .is_ok());
        assert!(tree
            .validate(ACCESSIBILITY_TOOL, &json!({"extra":1}))
            .is_err());
    }

    #[test]
    fn accessibility_tree_requires_a_screenshot_and_redacts_history() {
        let data = tempfile::tempdir().unwrap();
        let runtime = tree_runtime(data.path(), 0);
        let error = call(&runtime, ACCESSIBILITY_TOOL, json!({})).unwrap_err();
        assert_eq!(error.code, "computer_frame_required");

        call(&runtime, "computer_screenshot", json!({})).unwrap();
        let result = call(&runtime, ACCESSIBILITY_TOOL, json!({})).unwrap();
        let text = result.model_text.unwrap();
        assert!(text.starts_with(crate::OUTLINE_MARKER));
        // Boxes use the inverse of the screenshot's endpoint mapping
        // (input 0..3 onto pixels 0..7).
        assert!(
            text.contains("text_field \"Password\" focused password value=<redacted> [2,1,5,3]"),
            "{text}"
        );
        assert!(!text.contains("hunter2"));
        assert!(result.image.is_none());
        let value = result.value.to_string();
        assert_eq!(result.value["type"], "computer_accessibility");
        assert_eq!(result.value["nodeCount"], 2);
        for secret in ["hunter2", "Password", "Fixture", "Login"] {
            assert!(!value.contains(secret), "{secret} leaked into {value}");
        }
    }

    #[test]
    fn takeover_during_collection_returns_promptly() {
        let data = tempfile::tempdir().unwrap();
        let runtime = tree_runtime(data.path(), 10_000);
        call(&runtime, "computer_screenshot", json!({})).unwrap();
        let background = runtime.clone();
        let started = std::time::Instant::now();
        let handle = std::thread::spawn(move || call(&background, ACCESSIBILITY_TOOL, json!({})));
        std::thread::sleep(std::time::Duration::from_millis(100));
        // Must not wait behind the collector for the executor lock.
        runtime.take_user_control().unwrap();
        let error = handle.join().unwrap().unwrap_err();
        assert_eq!(error.code, "computer_control_unavailable");
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
    }

    #[test]
    fn a_new_screenshot_during_collection_discards_the_tree() {
        let data = tempfile::tempdir().unwrap();
        let runtime = tree_runtime(data.path(), 300);
        call(&runtime, "computer_screenshot", json!({})).unwrap();
        let background = runtime.clone();
        let handle = std::thread::spawn(move || call(&background, ACCESSIBILITY_TOOL, json!({})));
        std::thread::sleep(std::time::Duration::from_millis(50));
        call(&runtime, "computer_screenshot", json!({})).unwrap();
        let error = handle.join().unwrap().unwrap_err();
        assert_eq!(error.code, "computer_frame_stale");
    }
}
