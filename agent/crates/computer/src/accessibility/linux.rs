//! Linux (X11) collector over AT-SPI2 on the session accessibility bus.
//!
//! AT-SPI screen coordinates are X screen pixels, the same space the X11
//! input backend uses. Role names come from the non-localized
//! `GetRoleName`, which is stable across AT-SPI releases.

use super::{
    clean, walk, AccessibilityBackend, AccessibilityNode, AccessibilityRequest,
    AccessibilityRuntimeInfo, AccessibilitySnapshot, AccessibilityTarget, Budget, InputRect, Role,
    StateFlags, Visited, MAX_FIELD_CHARACTERS,
};
use crate::{ComputerError, ComputerResult, PermissionState};
use dbus::arg::{prop_cast, PropMap};
use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
use dbus::blocking::{BlockingSender, Connection};
use dbus::channel::Channel;
use dbus::strings::Path;
use dbus::Message;
use std::time::Duration;

const REGISTRY: &str = "org.a11y.atspi.Registry";
const ROOT: &str = "/org/a11y/atspi/accessible/root";
const ACCESSIBLE: &str = "org.a11y.atspi.Accessible";
const COMPONENT: &str = "org.a11y.atspi.Component";
const TEXT: &str = "org.a11y.atspi.Text";
const VALUE: &str = "org.a11y.atspi.Value";
const NULL_PATH: &str = "/org/a11y/atspi/null";
/// `ATSPI_COORD_TYPE_SCREEN`.
const COORD_SCREEN: u32 = 0;
/// Upper bound for one AT-SPI call so a hung application cannot stall the walk.
const CALL_TIMEOUT: Duration = Duration::from_millis(500);
const STATUS_TIMEOUT: Duration = Duration::from_millis(500);

/// `AtspiStateType` bit indexes.
mod state {
    pub const ACTIVE: u32 = 1;
    pub const CHECKED: u32 = 4;
    pub const COLLAPSED: u32 = 5;
    pub const EDITABLE: u32 = 7;
    pub const ENABLED: u32 = 8;
    pub const EXPANDABLE: u32 = 9;
    pub const EXPANDED: u32 = 10;
    pub const FOCUSED: u32 = 12;
    pub const SELECTED: u32 = 23;
    pub const SENSITIVE: u32 = 24;
    pub const SHOWING: u32 = 25;
    pub const INDETERMINATE: u32 = 32;
    pub const READ_ONLY: u32 = 43;
}

pub(super) struct AtspiAccessibility(());

impl AtspiAccessibility {
    /// AT-SPI availability can change at runtime, so the backend is always
    /// installed and reports bus state on each query.
    pub(super) fn new() -> Option<Self> {
        Some(Self(()))
    }
}

/// Whether the session accessibility bus is present and enabled.
pub(super) fn permission() -> PermissionState {
    let Ok(session) = Connection::new_session() else {
        return PermissionState::Unsupported;
    };
    let status = session.with_proxy("org.a11y.Bus", "/org/a11y/bus", STATUS_TIMEOUT);
    match status.get::<bool>("org.a11y.Status", "IsEnabled") {
        Ok(true) => PermissionState::Allowed,
        Ok(false) => PermissionState::Denied,
        Err(_) => PermissionState::Unsupported,
    }
}

impl AccessibilityBackend for AtspiAccessibility {
    fn runtime_info(&self) -> AccessibilityRuntimeInfo {
        let permission = permission();
        AccessibilityRuntimeInfo {
            available: permission != PermissionState::Unsupported,
            permission,
        }
    }

    fn snapshot(
        &self,
        request: &AccessibilityRequest,
        cancelled: &dyn Fn() -> bool,
    ) -> ComputerResult<AccessibilitySnapshot> {
        match permission() {
            PermissionState::Allowed => {}
            PermissionState::Denied => return Err(ComputerError::PermissionRequired),
            _ => {
                return Err(ComputerError::Backend(
                    "the accessibility bus is unavailable".into(),
                ))
            }
        }
        let bus = Bus::connect()?;
        let mut budget = Budget::new(request, cancelled);
        let mut snapshot = AccessibilitySnapshot::default();
        let Some((application, window)) = bus.active_window(&budget) else {
            return Ok(snapshot);
        };
        snapshot.application = bus.name(&application);
        snapshot.window = bus.name(&window);
        let root = match request.target {
            AccessibilityTarget::FrontmostApplication => application,
            AccessibilityTarget::FocusedWindow => window,
            AccessibilityTarget::Point { x, y } => match bus.at_point(&window, x, y) {
                Some(element) => element,
                None => return Ok(snapshot),
            },
        };
        let mut nodes = Vec::new();
        walk(root, request, &mut budget, &mut nodes, |element| {
            bus.describe(element, request.include_offscreen)
        })?;
        snapshot.nodes = nodes;
        snapshot.truncation = budget.truncation();
        Ok(snapshot)
    }
}

/// An accessible object reference: owning bus name and object path.
#[derive(Clone)]
struct Ref {
    bus: String,
    path: String,
}

/// A private connection to the accessibility bus.
struct Bus(Connection);

impl Bus {
    fn connect() -> ComputerResult<Self> {
        let unavailable = |_| ComputerError::Backend("the accessibility bus is unavailable".into());
        let session = Connection::new_session().map_err(unavailable)?;
        let (address,): (String,) = session
            .with_proxy("org.a11y.Bus", "/org/a11y/bus", STATUS_TIMEOUT)
            .method_call("org.a11y.Bus", "GetAddress", ())
            .map_err(unavailable)?;
        let mut channel = Channel::open_private(&address).map_err(unavailable)?;
        channel.register().map_err(unavailable)?;
        Ok(Self(Connection::from(channel)))
    }

    /// Sends one call and returns the reply, or `None` when the object is
    /// gone, refuses the interface, or times out.
    fn call(
        &self,
        target: &Ref,
        interface: &str,
        member: &str,
        build: impl FnOnce(Message) -> Message,
    ) -> Option<Message> {
        let message =
            Message::new_method_call(target.bus.as_str(), target.path.as_str(), interface, member)
                .ok()?;
        self.0
            .send_with_reply_and_block(build(message), CALL_TIMEOUT)
            .ok()
    }

    fn children(&self, target: &Ref) -> Vec<Ref> {
        let Some(reply) = self.call(target, ACCESSIBLE, "GetChildren", |message| message) else {
            return Vec::new();
        };
        reply
            .read1::<Vec<(&str, Path)>>()
            .map(|children| {
                children
                    .into_iter()
                    .filter(|(_, path)| &**path != NULL_PATH)
                    .map(|(bus, path)| Ref {
                        bus: bus.to_owned(),
                        path: path.to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn properties(&self, target: &Ref) -> Option<PropMap> {
        self.0
            .with_proxy(target.bus.as_str(), target.path.as_str(), CALL_TIMEOUT)
            .get_all(ACCESSIBLE)
            .ok()
    }

    fn name(&self, target: &Ref) -> Option<String> {
        self.properties(target).and_then(|properties| {
            prop_cast::<String>(&properties, "Name").and_then(|name| clean(name))
        })
    }

    fn states(&self, target: &Ref) -> States {
        self.call(target, ACCESSIBLE, "GetState", |message| message)
            .and_then(|reply| reply.read1::<Vec<u32>>().ok())
            .map_or(States(Vec::new()), States)
    }

    fn role_name(&self, target: &Ref) -> Option<String> {
        self.call(target, ACCESSIBLE, "GetRoleName", |message| message)
            .and_then(|reply| reply.read1::<String>().ok())
    }

    fn extents(&self, target: &Ref) -> Option<InputRect> {
        let reply = self.call(target, COMPONENT, "GetExtents", |message| {
            message.append1(COORD_SCREEN)
        })?;
        let (x, y, width, height) = reply.read1::<(i32, i32, i32, i32)>().ok()?;
        (width > 0 && height > 0).then(|| InputRect {
            x: f64::from(x),
            y: f64::from(y),
            width: f64::from(width),
            height: f64::from(height),
        })
    }

    /// The active top-level window and its application, if any.
    fn active_window(&self, budget: &Budget<'_>) -> Option<(Ref, Ref)> {
        let root = Ref {
            bus: REGISTRY.into(),
            path: ROOT.into(),
        };
        for application in self.children(&root) {
            if budget.remaining().is_zero() {
                break;
            }
            for window in self.children(&application) {
                if self.states(&window).contains(state::ACTIVE) {
                    return Some((application, window));
                }
            }
        }
        None
    }

    fn at_point(&self, window: &Ref, x: i32, y: i32) -> Option<Ref> {
        let reply = self.call(window, COMPONENT, "GetAccessibleAtPoint", |message| {
            message.append3(x, y, COORD_SCREEN)
        })?;
        let (bus, path) = reply.read1::<(&str, Path)>().ok()?;
        (&*path != NULL_PATH).then(|| Ref {
            bus: bus.to_owned(),
            path: path.to_string(),
        })
    }

    fn describe(&self, target: &Ref, include_offscreen: bool) -> Option<Visited<Ref>> {
        let states = self.states(target);
        if !include_offscreen && !states.0.is_empty() && !states.contains(state::SHOWING) {
            return None;
        }
        let role_name = self.role_name(target)?;
        let role = role(&role_name);
        let password = role_name == "password text";
        let properties = self.properties(target);
        let text = |key: &str| {
            properties
                .as_ref()
                .and_then(|properties| prop_cast::<String>(properties, key))
                .and_then(|value| clean(value))
        };
        let mut flags = StateFlags::empty();
        flags.set(StateFlags::PASSWORD, password);
        flags.set(StateFlags::FOCUSED, states.contains(state::FOCUSED));
        flags.set(
            StateFlags::DISABLED,
            !states.0.is_empty()
                && !states.contains(state::ENABLED)
                && !states.contains(state::SENSITIVE),
        );
        flags.set(StateFlags::SELECTED, states.contains(state::SELECTED));
        flags.set(StateFlags::CHECKED, states.contains(state::CHECKED));
        flags.set(StateFlags::MIXED, states.contains(state::INDETERMINATE));
        if states.contains(state::EXPANDED) {
            flags.insert(StateFlags::EXPANDED);
        } else if states.contains(state::COLLAPSED)
            || (states.contains(state::EXPANDABLE) && role.is_expandable())
        {
            flags.insert(StateFlags::COLLAPSED);
        }
        flags.set(
            StateFlags::EDITABLE,
            states.contains(state::EDITABLE) && !states.contains(state::READ_ONLY),
        );
        let value = if password {
            None
        } else {
            self.value(target, role)
        };
        Some(Visited {
            node: AccessibilityNode {
                depth: 0,
                role,
                name: text("Name"),
                value,
                description: text("Description"),
                states: flags,
                bounds: self.extents(target),
            },
            children: self.children(target),
        })
    }

    fn value(&self, target: &Ref, role: Role) -> Option<String> {
        match role {
            Role::TextField | Role::ComboBox => {
                let end = i32::try_from(MAX_FIELD_CHARACTERS + 1).unwrap_or(i32::MAX);
                self.call(target, TEXT, "GetText", |message| {
                    message.append2(0i32, end)
                })
                .and_then(|reply| reply.read1::<String>().ok())
                .and_then(|text| clean(&text))
            }
            Role::Slider | Role::ProgressBar | Role::ScrollBar => self
                .0
                .with_proxy(target.bus.as_str(), target.path.as_str(), CALL_TIMEOUT)
                .get::<f64>(VALUE, "CurrentValue")
                .ok()
                .map(|value| format!("{value:.2}")),
            _ => None,
        }
    }
}

/// An AT-SPI state set: bit `n` lives in word `n / 32`.
struct States(Vec<u32>);

impl States {
    fn contains(&self, bit: u32) -> bool {
        self.0
            .get((bit / 32) as usize)
            .is_some_and(|word| word & (1 << (bit % 32)) != 0)
    }
}

fn role(name: &str) -> Role {
    match name {
        "application" => Role::Application,
        "frame" | "window" => Role::Window,
        "dialog" | "alert" | "file chooser" | "color chooser" | "font chooser" => Role::Dialog,
        "panel" | "filler" | "section" | "grouping" | "split pane" | "viewport"
        | "layered pane" | "root pane" | "glass pane" | "internal frame" | "form"
        | "block quote" | "landmark" => Role::Group,
        "tool bar" | "status bar" => Role::Toolbar,
        "menu bar" => Role::MenuBar,
        "menu" | "popup menu" => Role::Menu,
        "menu item" | "check menu item" | "radio menu item" | "tearoff menu item" => Role::MenuItem,
        "push button" | "toggle button" | "button" | "push button menu" => Role::Button,
        "link" => Role::Link,
        "entry" | "text" | "password text" | "spin button" | "editbar" | "terminal" => {
            Role::TextField
        }
        "label" | "static" | "paragraph" | "caption" | "description value" => Role::Text,
        "heading" => Role::Heading,
        "check box" => Role::Checkbox,
        "radio button" => Role::Radio,
        "switch" => Role::Switch,
        "combo box" => Role::ComboBox,
        "list" | "list box" => Role::List,
        "list item" => Role::ListItem,
        "tree" | "tree table" => Role::Tree,
        "tree item" => Role::TreeItem,
        "page tab" => Role::Tab,
        "page tab list" => Role::TabList,
        "table" => Role::Table,
        "table row" => Role::Row,
        "table cell" => Role::Cell,
        "table column header" | "column header" | "table row header" | "row header" => {
            Role::ColumnHeader
        }
        "image" | "icon" | "image map" | "canvas" => Role::Image,
        "scroll pane" => Role::ScrollArea,
        "scroll bar" => Role::ScrollBar,
        "slider" => Role::Slider,
        "progress bar" | "level bar" => Role::ProgressBar,
        "document frame"
        | "document text"
        | "document spreadsheet"
        | "document presentation"
        | "document email" => Role::Document,
        "document web" => Role::WebArea,
        _ => Role::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_names_map_to_the_shared_vocabulary() {
        assert_eq!(role("push button"), Role::Button);
        assert_eq!(role("password text"), Role::TextField);
        assert_eq!(role("page tab"), Role::Tab);
        assert_eq!(role("something new"), Role::Unknown);
    }

    #[test]
    fn state_sets_span_two_words() {
        let states = States(vec![1 << state::FOCUSED, 1 << (state::READ_ONLY - 32)]);
        assert!(states.contains(state::FOCUSED));
        assert!(states.contains(state::READ_ONLY));
        assert!(!states.contains(state::SHOWING));
        assert!(!States(Vec::new()).contains(state::ACTIVE));
    }
}
