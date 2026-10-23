//! macOS collector over the Accessibility (AX) API.
//!
//! AX reports frames in global points with the origin at the top-left of the
//! main display. The executor's input space for the main display spans its
//! pixel dimensions, so frames are scaled by pixels-per-point here.

use super::{
    clean, walk, AccessibilityBackend, AccessibilityNode, AccessibilityRequest,
    AccessibilityRuntimeInfo, AccessibilitySnapshot, AccessibilityTarget, Budget, InputRect, Role,
    StateFlags, Visited,
};
use crate::{ComputerError, ComputerResult, PermissionState};
use core_foundation::array::{CFArray, CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
use core_foundation::base::{CFType, CFTypeRef, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionaryGetValue;
use core_foundation::number::CFNumber;
use core_foundation::number::{kCFNumberSInt32Type, CFNumberGetValue};
use core_foundation::string::{CFString, CFStringRef};
use std::ffi::c_void;
use std::time::Duration;

type AXUIElementRef = CFTypeRef;
type AXValueRef = CFTypeRef;
type AXError = i32;

const AX_SUCCESS: AXError = 0;
const AX_API_DISABLED: AXError = -25211;
const AX_CANNOT_COMPLETE: AXError = -25204;
const AX_VALUE_CGPOINT: u32 = 1;
const AX_VALUE_CGSIZE: u32 = 2;
/// Upper bound for one AX message so a hung target cannot stall the walk.
const MESSAGE_TIMEOUT: Duration = Duration::from_millis(500);

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct CGSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> u8;
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> AXError;
    fn AXUIElementCopyElementAtPosition(
        application: AXUIElementRef,
        x: f32,
        y: f32,
        element: *mut AXUIElementRef,
    ) -> AXError;
    fn AXUIElementSetMessagingTimeout(element: AXUIElementRef, seconds: f32) -> AXError;
    fn AXUIElementGetTypeID() -> usize;
    fn AXValueGetValue(value: AXValueRef, kind: u32, out: *mut c_void) -> u8;
    fn AXValueGetTypeID() -> usize;
    fn CGMainDisplayID() -> u32;
    fn CGDisplayBounds(display: u32) -> CGRect;
    fn CGDisplayPixelsWide(display: u32) -> usize;
    fn CGDisplayPixelsHigh(display: u32) -> usize;
    fn CGWindowListCopyWindowInfo(option: u32, relative_to: u32) -> CFArrayRef;
}

const WINDOW_LIST_ON_SCREEN_ONLY: u32 = 1 << 0;
const WINDOW_LIST_EXCLUDE_DESKTOP: u32 = 1 << 4;

pub(super) struct MacAccessibility(());

impl MacAccessibility {
    /// The first CoreGraphics query in a process opens the window-server
    /// connection, which can take seconds; pay it here rather than inside a
    /// collection deadline.
    pub(super) fn new() -> Self {
        let _ = Scale::main();
        Self(())
    }
}

/// Accessibility trust, without prompting.
pub(super) fn permission() -> PermissionState {
    // SAFETY: AXIsProcessTrusted has no preconditions and never prompts.
    if unsafe { AXIsProcessTrusted() } != 0 {
        PermissionState::Allowed
    } else {
        PermissionState::Denied
    }
}

impl AccessibilityBackend for MacAccessibility {
    fn runtime_info(&self) -> AccessibilityRuntimeInfo {
        AccessibilityRuntimeInfo {
            available: true,
            permission: permission(),
        }
    }

    fn snapshot(
        &self,
        request: &AccessibilityRequest,
        cancelled: &dyn Fn() -> bool,
    ) -> ComputerResult<AccessibilitySnapshot> {
        // Missing permission surfaces as kAXErrorAPIDisabled below; an
        // upfront trust query can take seconds on first use.
        let scale = Scale::main();
        // SAFETY: the system-wide element is created with a +1 retain count.
        let system =
            Element(unsafe { CFType::wrap_under_create_rule(AXUIElementCreateSystemWide()) });
        let mut budget = Budget::new(request, cancelled);
        system.set_timeout(budget.remaining());
        let mut snapshot = AccessibilitySnapshot::default();
        let point = match request.target {
            AccessibilityTarget::Point { x, y } => {
                Some(scale.to_points(f64::from(x), f64::from(y)))
            }
            _ => None,
        };
        let application = match focused_application(&system, point) {
            Ok(Some(application)) => application,
            Ok(None) => return Ok(snapshot),
            Err(AX_API_DISABLED) => return Err(ComputerError::PermissionRequired),
            Err(_) => {
                snapshot.inaccessible = true;
                return Ok(snapshot);
            }
        };
        application.set_timeout(budget.remaining());
        snapshot.application = application.string("AXTitle");
        let window = application.element("AXFocusedWindow").ok();
        snapshot.window = window.as_ref().and_then(|window| window.string("AXTitle"));
        let root = match (request.target, point) {
            (AccessibilityTarget::FrontmostApplication, _) => application,
            (AccessibilityTarget::Point { .. }, Some(point)) => {
                match application.element_at(point.x as f32, point.y as f32) {
                    Ok(element) => element,
                    Err(AX_API_DISABLED) => return Err(ComputerError::PermissionRequired),
                    Err(_) => {
                        snapshot.inaccessible = true;
                        return Ok(snapshot);
                    }
                }
            }
            _ => window.unwrap_or(application),
        };
        let mut nodes = Vec::new();
        walk(root, request, &mut budget, &mut nodes, |element| {
            describe(element, &scale, &budget_timeout(request))
        })?;
        snapshot.nodes = nodes;
        snapshot.truncation = budget.truncation();
        Ok(snapshot)
    }
}

/// The application to inspect: the one owning the window under `point`, or
/// else the focused one. `Ok(None)` means no application window is visible.
///
/// The system-wide focus query fails for some host processes, so the front
/// normal-layer window's owner from the window server is the fallback.
fn focused_application(
    system: &Element,
    point: Option<CGPoint>,
) -> Result<Option<Element>, AXError> {
    if point.is_none() {
        match system.element("AXFocusedApplication") {
            Ok(application) => return Ok(Some(application)),
            Err(AX_API_DISABLED) => return Err(AX_API_DISABLED),
            Err(_) => {}
        }
    }
    let Some(pid) = front_window_owner(point) else {
        return Ok(None);
    };
    // SAFETY: the application element is created with a +1 retain count.
    let application = unsafe { AXUIElementCreateApplication(pid) };
    if application.is_null() {
        return Err(AX_CANNOT_COMPLETE);
    }
    // SAFETY: `application` is non-null and owned by this call.
    Ok(Some(Element(unsafe {
        CFType::wrap_under_create_rule(application)
    })))
}

/// Owner PID of the frontmost normal-layer window, optionally the first one
/// containing `point` (in global points).
fn front_window_owner(point: Option<CGPoint>) -> Option<i32> {
    // SAFETY: the window list is returned with a +1 retain count and wrapped
    // so it is released; dictionary and number reads borrow from it.
    unsafe {
        let list =
            CGWindowListCopyWindowInfo(WINDOW_LIST_ON_SCREEN_ONLY | WINDOW_LIST_EXCLUDE_DESKTOP, 0);
        if list.is_null() {
            return None;
        }
        let _owned = CFType::wrap_under_create_rule(list.cast());
        let layer_key = CFString::from_static_string("kCGWindowLayer");
        let owner_key = CFString::from_static_string("kCGWindowOwnerPID");
        let bounds_key = CFString::from_static_string("kCGWindowBounds");
        let number = |window: *const c_void, key: &CFString| {
            let value = CFDictionaryGetValue(window.cast(), key.as_CFTypeRef());
            let mut out = 0i32;
            (!value.is_null()
                && CFNumberGetValue(value.cast(), kCFNumberSInt32Type, (&raw mut out).cast()))
            .then_some(out)
        };
        let bound = |window: *const c_void, key: &str| {
            let bounds = CFDictionaryGetValue(window.cast(), bounds_key.as_CFTypeRef());
            if bounds.is_null() {
                return None;
            }
            let key = CFString::new(key);
            let value = CFDictionaryGetValue(bounds.cast(), key.as_CFTypeRef());
            let mut out = 0f64;
            (!value.is_null()
                && CFNumberGetValue(
                    value.cast(),
                    core_foundation::number::kCFNumberFloat64Type,
                    (&raw mut out).cast(),
                ))
            .then_some(out)
        };
        for index in 0..CFArrayGetCount(list) {
            let window = CFArrayGetValueAtIndex(list, index);
            if window.is_null() || number(window, &layer_key) != Some(0) {
                continue;
            }
            if let Some(point) = point {
                let (Some(x), Some(y), Some(width), Some(height)) = (
                    bound(window, "X"),
                    bound(window, "Y"),
                    bound(window, "Width"),
                    bound(window, "Height"),
                ) else {
                    continue;
                };
                if point.x < x || point.y < y || point.x >= x + width || point.y >= y + height {
                    continue;
                }
            }
            return number(window, &owner_key);
        }
        None
    }
}

fn budget_timeout(request: &AccessibilityRequest) -> Duration {
    request
        .deadline
        .saturating_duration_since(std::time::Instant::now())
        .min(MESSAGE_TIMEOUT)
}

fn describe(element: &Element, scale: &Scale, timeout: &Duration) -> Option<Visited<Element>> {
    element.set_timeout(*timeout);
    let role_name = element.string("AXRole")?;
    let subrole = element.string("AXSubrole");
    let mut states = StateFlags::empty();
    let secure = subrole.as_deref() == Some("AXSecureTextField");
    states.set(StateFlags::PASSWORD, secure);
    states.set(
        StateFlags::FOCUSED,
        element.boolean("AXFocused") == Some(true),
    );
    states.set(
        StateFlags::DISABLED,
        element.boolean("AXEnabled") == Some(false),
    );
    states.set(
        StateFlags::SELECTED,
        element.boolean("AXSelected") == Some(true),
    );
    let role = role(&role_name, subrole.as_deref());
    match element.boolean("AXExpanded") {
        Some(true) => states.insert(StateFlags::EXPANDED),
        // Many controls report a false value without being expandable.
        Some(false) if role.is_expandable() => states.insert(StateFlags::COLLAPSED),
        _ => {}
    }
    let value = if secure {
        None
    } else {
        match role {
            Role::Checkbox | Role::Radio | Role::Switch => {
                match element.number("AXValue").map(|value| value as i64) {
                    Some(1) => states.insert(StateFlags::CHECKED),
                    Some(2) => states.insert(StateFlags::MIXED),
                    _ => {}
                }
                None
            }
            _ => element.value_text(),
        }
    };
    if role == Role::TextField {
        states.set(StateFlags::EDITABLE, !states.contains(StateFlags::DISABLED));
    }
    let name = element
        .string("AXTitle")
        .or_else(|| element.string("AXDescription"))
        .or_else(|| {
            element
                .element("AXTitleUIElement")
                .ok()
                .and_then(|title| title.string("AXValue"))
        });
    let description = element.string("AXHelp").or_else(|| {
        element
            .string("AXRoleDescription")
            .filter(|_| role == Role::Unknown)
    });
    let bounds = element.frame().map(|frame| scale.to_input(frame));
    let children = element.children("AXChildren");
    Some(Visited {
        node: AccessibilityNode {
            depth: 0,
            role,
            name,
            value,
            description,
            states,
            bounds,
        },
        children,
    })
}

fn role(role: &str, subrole: Option<&str>) -> Role {
    match (role, subrole) {
        (_, Some("AXSecureTextField")) => Role::TextField,
        (_, Some("AXSwitch")) => Role::Switch,
        (_, Some("AXTabButton")) => Role::Tab,
        (_, Some("AXDialog" | "AXSystemDialog" | "AXFloatingWindow")) => Role::Dialog,
        ("AXApplication", _) => Role::Application,
        ("AXWindow" | "AXSheet" | "AXDrawer", _) => Role::Window,
        ("AXGroup" | "AXSplitGroup" | "AXLayoutArea" | "AXSplitter", _) => Role::Group,
        ("AXToolbar", _) => Role::Toolbar,
        ("AXMenuBar", _) => Role::MenuBar,
        ("AXMenu", _) => Role::Menu,
        ("AXMenuItem" | "AXMenuBarItem", _) => Role::MenuItem,
        ("AXButton" | "AXMenuButton" | "AXPopUpButton" | "AXDisclosureTriangle", _) => Role::Button,
        ("AXLink", _) => Role::Link,
        ("AXTextField" | "AXTextArea" | "AXSearchField", _) => Role::TextField,
        ("AXStaticText", _) => Role::Text,
        ("AXHeading", _) => Role::Heading,
        ("AXCheckBox", _) => Role::Checkbox,
        ("AXRadioButton", _) => Role::Radio,
        ("AXComboBox", _) => Role::ComboBox,
        ("AXList", _) => Role::List,
        ("AXOutline", _) => Role::Tree,
        ("AXTabGroup", _) => Role::TabList,
        ("AXTable", _) => Role::Table,
        ("AXRow", Some("AXOutlineRow")) => Role::TreeItem,
        ("AXRow", _) => Role::Row,
        ("AXCell", _) => Role::Cell,
        ("AXColumn" | "AXSortButton", _) => Role::ColumnHeader,
        ("AXImage", _) => Role::Image,
        ("AXScrollArea", _) => Role::ScrollArea,
        ("AXScrollBar", _) => Role::ScrollBar,
        ("AXSlider", _) => Role::Slider,
        ("AXProgressIndicator" | "AXBusyIndicator", _) => Role::ProgressBar,
        ("AXWebArea", _) => Role::WebArea,
        ("AXBrowser" | "AXGrid", _) => Role::Table,
        ("AXRadioGroup", _) => Role::Group,
        _ => Role::Unknown,
    }
}

/// Main-display pixels per AX point.
struct Scale {
    x: f64,
    y: f64,
}

impl Scale {
    fn main() -> Self {
        // SAFETY: CoreGraphics display queries have no preconditions.
        let (bounds, width, height) = unsafe {
            let display = CGMainDisplayID();
            (
                CGDisplayBounds(display),
                CGDisplayPixelsWide(display),
                CGDisplayPixelsHigh(display),
            )
        };
        let ratio = |pixels: usize, points: f64| {
            if points > 0.0 && pixels > 0 {
                pixels as f64 / points
            } else {
                1.0
            }
        };
        Self {
            x: ratio(width, bounds.size.width),
            y: ratio(height, bounds.size.height),
        }
    }

    fn to_input(&self, frame: CGRect) -> InputRect {
        InputRect {
            x: frame.origin.x * self.x,
            y: frame.origin.y * self.y,
            width: frame.size.width * self.x,
            height: frame.size.height * self.y,
        }
    }

    fn to_points(&self, x: f64, y: f64) -> CGPoint {
        CGPoint {
            x: x / self.x,
            y: y / self.y,
        }
    }
}

/// A retained `AXUIElementRef`.
struct Element(CFType);

impl Element {
    fn raw(&self) -> AXUIElementRef {
        self.0.as_CFTypeRef()
    }

    fn set_timeout(&self, timeout: Duration) {
        let seconds = timeout.as_secs_f32().max(0.01);
        // SAFETY: `raw` is a live AXUIElement for the lifetime of `self`.
        unsafe {
            AXUIElementSetMessagingTimeout(self.raw(), seconds);
        }
    }

    fn copy(&self, attribute: &'static str) -> Result<CFType, AXError> {
        let name = CFString::from_static_string(attribute);
        let mut value: CFTypeRef = std::ptr::null();
        // SAFETY: on success AX returns a +1 retained value in `value`.
        let status = unsafe {
            AXUIElementCopyAttributeValue(self.raw(), name.as_concrete_TypeRef(), &mut value)
        };
        if status != AX_SUCCESS {
            return Err(status);
        }
        if value.is_null() {
            return Err(AX_CANNOT_COMPLETE);
        }
        // SAFETY: `value` is non-null and owned by this call.
        Ok(unsafe { CFType::wrap_under_create_rule(value) })
    }

    fn element(&self, attribute: &'static str) -> Result<Self, AXError> {
        let value = self.copy(attribute)?;
        // SAFETY: AXUIElementGetTypeID has no preconditions.
        if value.type_of() == unsafe { AXUIElementGetTypeID() } {
            Ok(Self(value))
        } else {
            Err(AX_CANNOT_COMPLETE)
        }
    }

    fn element_at(&self, x: f32, y: f32) -> Result<Self, AXError> {
        let mut element: AXUIElementRef = std::ptr::null();
        // SAFETY: on success AX returns a +1 retained element.
        let status = unsafe { AXUIElementCopyElementAtPosition(self.raw(), x, y, &mut element) };
        if status != AX_SUCCESS {
            return Err(status);
        }
        if element.is_null() {
            return Err(AX_CANNOT_COMPLETE);
        }
        // SAFETY: `element` is non-null and owned by this call.
        Ok(Self(unsafe { CFType::wrap_under_create_rule(element) }))
    }

    fn string(&self, attribute: &'static str) -> Option<String> {
        let value = self.copy(attribute).ok()?.downcast_into::<CFString>()?;
        clean(&value.to_string())
    }

    fn boolean(&self, attribute: &'static str) -> Option<bool> {
        let value = self.copy(attribute).ok()?;
        if let Some(value) = value.downcast::<CFBoolean>() {
            return Some(value.into());
        }
        value
            .downcast::<CFNumber>()
            .and_then(|number| number.to_i64())
            .map(|number| number != 0)
    }

    fn number(&self, attribute: &'static str) -> Option<f64> {
        self.copy(attribute).ok()?.downcast::<CFNumber>()?.to_f64()
    }

    /// `AXValue` rendered as text when it is a string or number.
    fn value_text(&self) -> Option<String> {
        let value = self.copy("AXValue").ok()?;
        if let Some(text) = value.downcast::<CFString>() {
            return clean(&text.to_string());
        }
        let number = value.downcast::<CFNumber>()?;
        number
            .to_i64()
            .map(|value| value.to_string())
            .or_else(|| number.to_f64().map(|value| format!("{value:.2}")))
    }

    fn frame(&self) -> Option<CGRect> {
        let mut point = CGPoint::default();
        let mut size = CGSize::default();
        let position = self.copy("AXPosition").ok()?;
        let extent = self.copy("AXSize").ok()?;
        // SAFETY: AXValueGetTypeID has no preconditions; the values are
        // checked to be AXValues before reading into matching C structs.
        unsafe {
            let ax_value = AXValueGetTypeID();
            if position.type_of() != ax_value || extent.type_of() != ax_value {
                return None;
            }
            if AXValueGetValue(
                position.as_CFTypeRef(),
                AX_VALUE_CGPOINT,
                (&raw mut point).cast(),
            ) == 0
                || AXValueGetValue(
                    extent.as_CFTypeRef(),
                    AX_VALUE_CGSIZE,
                    (&raw mut size).cast(),
                ) == 0
            {
                return None;
            }
        }
        Some(CGRect {
            origin: point,
            size,
        })
    }

    fn children(&self, attribute: &'static str) -> Vec<Self> {
        let Some(array) = self
            .copy(attribute)
            .ok()
            .and_then(|value| value.downcast_into::<CFArray<*const c_void>>())
        else {
            return Vec::new();
        };
        // SAFETY: AXUIElementGetTypeID has no preconditions.
        let element_type = unsafe { AXUIElementGetTypeID() };
        array
            .iter()
            .filter(|item| !item.is_null())
            // SAFETY: array items are borrowed CF objects; retain each one.
            .map(|item| unsafe { CFType::wrap_under_get_rule(*item) })
            .filter(|item| item.type_of() == element_type)
            .map(Self)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_map_to_the_shared_vocabulary() {
        assert_eq!(role("AXButton", None), Role::Button);
        assert_eq!(
            role("AXTextField", Some("AXSecureTextField")),
            Role::TextField
        );
        assert_eq!(role("AXRow", Some("AXOutlineRow")), Role::TreeItem);
        assert_eq!(role("AXWindow", Some("AXDialog")), Role::Dialog);
        assert_eq!(role("AXSomethingNew", None), Role::Unknown);
    }

    #[test]
    fn retina_scale_maps_points_to_input_pixels() {
        let scale = Scale { x: 2.0, y: 2.0 };
        let input = scale.to_input(CGRect {
            origin: CGPoint { x: 10.0, y: 20.0 },
            size: CGSize {
                width: 30.0,
                height: 40.0,
            },
        });
        assert_eq!(
            (input.x, input.y, input.width, input.height),
            (20.0, 40.0, 60.0, 80.0)
        );
        let point = scale.to_points(20.0, 40.0);
        assert_eq!((point.x, point.y), (10.0, 20.0));
    }

    /// Requires Accessibility permission; run manually on a desktop session.
    #[test]
    #[ignore = "requires a macOS desktop session with Accessibility permission"]
    fn collects_the_focused_window() {
        let backend = MacAccessibility::new();
        let request = AccessibilityRequest {
            target: AccessibilityTarget::FocusedWindow,
            max_depth: 8,
            max_nodes: 200,
            include_offscreen: false,
            display: InputRect {
                x: 0.0,
                y: 0.0,
                width: 1.0e5,
                height: 1.0e5,
            },
            deadline: std::time::Instant::now() + Duration::from_secs(2),
        };
        let snapshot = backend.snapshot(&request, &|| false).unwrap();
        eprintln!(
            "nodes={} truncation={:?} inaccessible={}",
            snapshot.nodes.len(),
            snapshot.truncation,
            snapshot.inaccessible
        );
        assert!(snapshot.application.is_some());
        assert!(!snapshot.nodes.is_empty());
        assert_eq!(snapshot.truncation, None);
    }
}
