//! Windows collector over UI Automation.
//!
//! UIA reports physical-pixel screen rectangles only when the calling thread
//! is per-monitor DPI aware, which matches the executor's input space. Each
//! level is fetched with one cached `FindAllBuildCache` call so a walk costs
//! one cross-process round trip per expanded element.

// UIA constants keep their Windows SDK names in match patterns.
#![allow(non_upper_case_globals)]

use super::{
    clean, walk, AccessibilityBackend, AccessibilityNode, AccessibilityRequest,
    AccessibilityRuntimeInfo, AccessibilitySnapshot, AccessibilityTarget, Budget, InputRect, Role,
    StateFlags, Visited,
};
use crate::{ComputerError, ComputerResult, PermissionState};
use std::time::Duration;
use windows::core::{Interface, BSTR, HRESULT};
use windows::Win32::Foundation::{CloseHandle, E_ACCESSDENIED, HWND, POINT, RECT};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Accessibility::*;
use windows::Win32::UI::HiDpi::{
    SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::WindowsAndMessaging::{GetAncestor, GetForegroundWindow, GA_ROOT};

/// Upper bound for one UIA request so a hung provider cannot stall the walk.
const TRANSACTION_TIMEOUT: Duration = Duration::from_millis(500);
const ELEMENT_NOT_AVAILABLE: HRESULT = HRESULT(UIA_E_ELEMENTNOTAVAILABLE as i32);
const UIA_TIMEOUT: HRESULT = HRESULT(UIA_E_TIMEOUT as i32);

/// Properties fetched for every element.
const PROPERTIES: [UIA_PROPERTY_ID; 9] = [
    UIA_NamePropertyId,
    UIA_ControlTypePropertyId,
    UIA_BoundingRectanglePropertyId,
    UIA_IsPasswordPropertyId,
    UIA_IsEnabledPropertyId,
    UIA_HasKeyboardFocusPropertyId,
    UIA_IsOffscreenPropertyId,
    UIA_HelpTextPropertyId,
    UIA_ProcessIdPropertyId,
];

/// Patterns fetched for every element; unsupported ones are simply absent.
const PATTERNS: [UIA_PATTERN_ID; 4] = [
    UIA_ValuePatternId,
    UIA_TogglePatternId,
    UIA_ExpandCollapsePatternId,
    UIA_SelectionItemPatternId,
];

pub(super) struct UiaAccessibility(());

impl UiaAccessibility {
    pub(super) fn new() -> Self {
        Self(())
    }
}

impl AccessibilityBackend for UiaAccessibility {
    fn runtime_info(&self) -> AccessibilityRuntimeInfo {
        // UI Automation needs no user-granted permission; elevated targets
        // are reported per snapshot as inaccessible.
        AccessibilityRuntimeInfo {
            available: true,
            permission: PermissionState::Allowed,
        }
    }

    fn snapshot(
        &self,
        request: &AccessibilityRequest,
        cancelled: &dyn Fn() -> bool,
    ) -> ComputerResult<AccessibilitySnapshot> {
        let _dpi = DpiAwareness::per_monitor();
        let com = ComApartment::enter()?;
        let mut budget = Budget::new(request, cancelled);
        // SAFETY: COM is initialized on this thread and the session keeps
        // the apartment alive until every interface it hands out is dropped.
        let session = unsafe { Session::new(com, budget.remaining()) }.map_err(backend)?;
        let mut snapshot = AccessibilitySnapshot::default();
        let root = match unsafe { session.root(request.target) } {
            Ok(Some(root)) => root,
            Ok(None) => return Ok(snapshot),
            Err(error) if is_inaccessible(&error) => {
                snapshot.inaccessible = true;
                return Ok(snapshot);
            }
            Err(error) => return Err(backend(error)),
        };
        // SAFETY: cached values are read from an element built with the
        // session's cache request.
        unsafe {
            snapshot.window = cached_string(root.CachedName());
            snapshot.application = root
                .CachedProcessId()
                .ok()
                .and_then(|pid| process_name(pid as u32));
        }
        let mut nodes = Vec::new();
        let walked = walk(root, request, &mut budget, &mut nodes, |element| {
            // SAFETY: as above; children are fetched with the same request.
            unsafe { session.describe(element) }
        });
        drop(session);
        walked?;
        snapshot.nodes = nodes;
        snapshot.truncation = budget.truncation();
        Ok(snapshot)
    }
}

/// One UI Automation client plus its cache request. Fields drop in order,
/// so the COM apartment is released after every interface.
struct Session {
    automation: IUIAutomation,
    cache: IUIAutomationCacheRequest,
    children: IUIAutomationCondition,
    _com: ComApartment,
}

impl Session {
    unsafe fn new(com: ComApartment, timeout: Duration) -> windows::core::Result<Self> {
        let automation: IUIAutomation =
            CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)
                .or_else(|_| CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER))?;
        if let Ok(automation) = automation.cast::<IUIAutomation2>() {
            let milliseconds = |duration: Duration| {
                u32::try_from(duration.as_millis())
                    .unwrap_or(u32::MAX)
                    .max(1)
            };
            let _ = automation.SetConnectionTimeout(milliseconds(timeout));
            let _ =
                automation.SetTransactionTimeout(milliseconds(timeout.min(TRANSACTION_TIMEOUT)));
        }
        let cache = automation.CreateCacheRequest()?;
        for property in PROPERTIES {
            cache.AddProperty(property)?;
        }
        for pattern in PATTERNS {
            cache.AddPattern(pattern)?;
        }
        // The control view hides layout-only elements, matching what a user
        // perceives as controls.
        let children = automation.ControlViewCondition()?;
        cache.SetTreeFilter(&children)?;
        Ok(Self {
            automation,
            cache,
            children,
            _com: com,
        })
    }

    unsafe fn root(
        &self,
        target: AccessibilityTarget,
    ) -> windows::core::Result<Option<IUIAutomationElement>> {
        let element = match target {
            AccessibilityTarget::Point { x, y } => {
                self.automation.ElementFromPoint(POINT { x, y })?
            }
            AccessibilityTarget::FocusedWindow | AccessibilityTarget::FrontmostApplication => {
                let window = GetForegroundWindow();
                if window.is_invalid() {
                    return Ok(None);
                }
                // Top-level window; UIA has no separate application object.
                let window: HWND = GetAncestor(window, GA_ROOT);
                self.automation.ElementFromHandle(window)?
            }
        };
        element.BuildUpdatedCache(&self.cache).map(Some)
    }

    unsafe fn describe(
        &self,
        element: &IUIAutomationElement,
    ) -> Option<Visited<IUIAutomationElement>> {
        let control = element.CachedControlType().ok()?;
        let role = role(control);
        let mut states = StateFlags::empty();
        let password = flag(element.CachedIsPassword());
        states.set(StateFlags::PASSWORD, password);
        states.set(StateFlags::FOCUSED, flag(element.CachedHasKeyboardFocus()));
        states.set(
            StateFlags::DISABLED,
            element
                .CachedIsEnabled()
                .is_ok_and(|enabled| !enabled.as_bool()),
        );
        let mut value = None;
        if let Ok(pattern) =
            element.GetCachedPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId)
        {
            if !password {
                value = cached_string(pattern.CachedValue());
            }
            let read_only = pattern
                .CachedIsReadOnly()
                .is_ok_and(|read_only| read_only.as_bool());
            states.set(
                StateFlags::EDITABLE,
                !read_only && matches!(role, Role::TextField | Role::ComboBox),
            );
        }
        if let Ok(pattern) =
            element.GetCachedPatternAs::<IUIAutomationTogglePattern>(UIA_TogglePatternId)
        {
            match pattern.CachedToggleState() {
                Ok(ToggleState_On) => states.insert(StateFlags::CHECKED),
                Ok(ToggleState_Indeterminate) => states.insert(StateFlags::MIXED),
                _ => {}
            }
        }
        if let Ok(pattern) = element
            .GetCachedPatternAs::<IUIAutomationExpandCollapsePattern>(UIA_ExpandCollapsePatternId)
        {
            match pattern.CachedExpandCollapseState() {
                Ok(ExpandCollapseState_Expanded | ExpandCollapseState_PartiallyExpanded) => {
                    states.insert(StateFlags::EXPANDED);
                }
                Ok(ExpandCollapseState_Collapsed) => states.insert(StateFlags::COLLAPSED),
                _ => {}
            }
        }
        if let Ok(pattern) = element
            .GetCachedPatternAs::<IUIAutomationSelectionItemPattern>(UIA_SelectionItemPatternId)
        {
            states.set(StateFlags::SELECTED, flag(pattern.CachedIsSelected()));
        }
        let bounds = element.CachedBoundingRectangle().ok().and_then(rect);
        if flag(element.CachedIsOffscreen()) {
            states.insert(StateFlags::OFFSCREEN);
        }
        let children = element
            .FindAllBuildCache(TreeScope_Children, &self.children, &self.cache)
            .ok()
            .map(|array| {
                let length = array.Length().unwrap_or(0);
                (0..length)
                    .filter_map(|index| array.GetElement(index).ok())
                    .collect()
            })
            .unwrap_or_default();
        Some(Visited {
            node: AccessibilityNode {
                depth: 0,
                role,
                name: cached_string(element.CachedName()),
                value,
                description: cached_string(element.CachedHelpText()),
                states,
                bounds,
            },
            children,
        })
    }
}

fn role(control: UIA_CONTROLTYPE_ID) -> Role {
    match control {
        UIA_ButtonControlTypeId | UIA_SplitButtonControlTypeId => Role::Button,
        UIA_CheckBoxControlTypeId => Role::Checkbox,
        UIA_ComboBoxControlTypeId => Role::ComboBox,
        UIA_EditControlTypeId => Role::TextField,
        UIA_HyperlinkControlTypeId => Role::Link,
        UIA_ImageControlTypeId => Role::Image,
        UIA_ListControlTypeId => Role::List,
        UIA_ListItemControlTypeId | UIA_DataItemControlTypeId => Role::ListItem,
        UIA_MenuControlTypeId => Role::Menu,
        UIA_MenuBarControlTypeId => Role::MenuBar,
        UIA_MenuItemControlTypeId => Role::MenuItem,
        UIA_ProgressBarControlTypeId => Role::ProgressBar,
        UIA_RadioButtonControlTypeId => Role::Radio,
        UIA_ScrollBarControlTypeId => Role::ScrollBar,
        UIA_SliderControlTypeId | UIA_SpinnerControlTypeId => Role::Slider,
        UIA_TabControlTypeId => Role::TabList,
        UIA_TabItemControlTypeId => Role::Tab,
        UIA_TextControlTypeId => Role::Text,
        UIA_ToolBarControlTypeId | UIA_StatusBarControlTypeId | UIA_AppBarControlTypeId => {
            Role::Toolbar
        }
        UIA_TreeControlTypeId => Role::Tree,
        UIA_TreeItemControlTypeId => Role::TreeItem,
        UIA_GroupControlTypeId | UIA_PaneControlTypeId | UIA_TitleBarControlTypeId => Role::Group,
        UIA_WindowControlTypeId => Role::Window,
        UIA_DocumentControlTypeId => Role::Document,
        UIA_TableControlTypeId | UIA_DataGridControlTypeId | UIA_CalendarControlTypeId => {
            Role::Table
        }
        UIA_HeaderControlTypeId => Role::Row,
        UIA_HeaderItemControlTypeId => Role::ColumnHeader,
        _ => Role::Unknown,
    }
}

fn flag(value: windows::core::Result<windows::core::BOOL>) -> bool {
    value.is_ok_and(|value| value.as_bool())
}

fn cached_string(value: windows::core::Result<BSTR>) -> Option<String> {
    value.ok().and_then(|value| clean(&value.to_string()))
}

fn rect(value: RECT) -> Option<InputRect> {
    let width = value.right.checked_sub(value.left)?;
    let height = value.bottom.checked_sub(value.top)?;
    (width > 0 && height > 0).then(|| InputRect {
        x: f64::from(value.left),
        y: f64::from(value.top),
        width: f64::from(width),
        height: f64::from(height),
    })
}

/// The executable's file stem, used as the application name.
fn process_name(pid: u32) -> Option<String> {
    // SAFETY: the handle is closed before returning; the buffer outlives
    // the call and `size` reports its capacity in UTF-16 units.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = [0u16; 512];
        let mut size = buffer.len() as u32;
        let result = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(process);
        result.ok()?;
        let path = String::from_utf16_lossy(&buffer[..size as usize]);
        let stem = std::path::Path::new(&path)
            .file_stem()?
            .to_string_lossy()
            .into_owned();
        clean(&stem)
    }
}

fn is_inaccessible(error: &windows::core::Error) -> bool {
    matches!(
        error.code(),
        E_ACCESSDENIED | ELEMENT_NOT_AVAILABLE | UIA_TIMEOUT
    )
}

fn backend(error: windows::core::Error) -> ComputerError {
    // HRESULTs only; never element text.
    ComputerError::Backend(format!(
        "UI Automation failed with {:#010x}",
        error.code().0
    ))
}

/// Multithreaded COM apartment for the collector thread.
struct ComApartment(bool);

impl ComApartment {
    fn enter() -> ComputerResult<Self> {
        // SAFETY: balanced with CoUninitialize in Drop when it succeeds.
        let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if result.is_err() {
            return Err(ComputerError::Backend(format!(
                "COM initialization failed with {:#010x}",
                result.0
            )));
        }
        Ok(Self(true))
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        if self.0 {
            // SAFETY: paired with the successful CoInitializeEx above.
            unsafe { CoUninitialize() };
        }
    }
}

/// Per-monitor DPI awareness for the collector thread.
struct DpiAwareness(DPI_AWARENESS_CONTEXT);

impl DpiAwareness {
    fn per_monitor() -> Self {
        // SAFETY: restores the previous thread context in Drop.
        Self(unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) })
    }
}

impl Drop for DpiAwareness {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            // SAFETY: restores the context returned when the guard was made.
            unsafe { SetThreadDpiAwarenessContext(self.0) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_types_map_to_the_shared_vocabulary() {
        assert_eq!(role(UIA_ButtonControlTypeId), Role::Button);
        assert_eq!(role(UIA_EditControlTypeId), Role::TextField);
        assert_eq!(role(UIA_TabItemControlTypeId), Role::Tab);
        assert_eq!(role(UIA_CONTROLTYPE_ID(1)), Role::Unknown);
    }

    #[test]
    fn rectangles_reject_empty_and_inverted_bounds() {
        assert!(rect(RECT {
            left: 10,
            top: 10,
            right: 10,
            bottom: 20
        })
        .is_none());
        assert!(rect(RECT {
            left: 10,
            top: 10,
            right: 5,
            bottom: 20
        })
        .is_none());
        let bounds = rect(RECT {
            left: -5,
            top: 2,
            right: 15,
            bottom: 12,
        })
        .unwrap();
        assert_eq!((bounds.x, bounds.width, bounds.height), (-5.0, 20.0, 10.0));
    }
}
