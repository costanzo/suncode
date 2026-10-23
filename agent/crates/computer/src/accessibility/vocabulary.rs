//! Provider-neutral roles and states shared by every platform collector.

/// Normalized element role. Platform roles without a close match become
/// [`Role::Unknown`] rather than leaking platform names to the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Application,
    Window,
    Dialog,
    Group,
    Toolbar,
    MenuBar,
    Menu,
    MenuItem,
    Button,
    Link,
    TextField,
    Text,
    Heading,
    Checkbox,
    Radio,
    Switch,
    ComboBox,
    List,
    ListItem,
    Tree,
    TreeItem,
    Tab,
    TabList,
    Table,
    Row,
    Cell,
    ColumnHeader,
    Image,
    ScrollArea,
    ScrollBar,
    Slider,
    ProgressBar,
    Document,
    WebArea,
    Unknown,
}

impl Role {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::Window => "window",
            Self::Dialog => "dialog",
            Self::Group => "group",
            Self::Toolbar => "toolbar",
            Self::MenuBar => "menu_bar",
            Self::Menu => "menu",
            Self::MenuItem => "menu_item",
            Self::Button => "button",
            Self::Link => "link",
            Self::TextField => "text_field",
            Self::Text => "text",
            Self::Heading => "heading",
            Self::Checkbox => "checkbox",
            Self::Radio => "radio",
            Self::Switch => "switch",
            Self::ComboBox => "combo_box",
            Self::List => "list",
            Self::ListItem => "list_item",
            Self::Tree => "tree",
            Self::TreeItem => "tree_item",
            Self::Tab => "tab",
            Self::TabList => "tab_list",
            Self::Table => "table",
            Self::Row => "row",
            Self::Cell => "cell",
            Self::ColumnHeader => "column_header",
            Self::Image => "image",
            Self::ScrollArea => "scroll_area",
            Self::ScrollBar => "scroll_bar",
            Self::Slider => "slider",
            Self::ProgressBar => "progress_bar",
            Self::Document => "document",
            Self::WebArea => "web_area",
            Self::Unknown => "unknown",
        }
    }

    /// Roles for which a collapsed state is meaningful.
    pub const fn is_expandable(self) -> bool {
        matches!(
            self,
            Self::TreeItem | Self::Row | Self::ComboBox | Self::MenuItem | Self::Group
        )
    }

    /// Containers that add little when they carry no name of their own.
    pub const fn is_structural(self) -> bool {
        matches!(self, Self::Group | Self::Unknown | Self::ScrollArea)
    }
}

/// Normalized element state bit set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct StateFlags(u16);

impl StateFlags {
    pub const FOCUSED: Self = Self(1 << 0);
    pub const DISABLED: Self = Self(1 << 1);
    pub const SELECTED: Self = Self(1 << 2);
    pub const CHECKED: Self = Self(1 << 3);
    pub const MIXED: Self = Self(1 << 4);
    pub const EXPANDED: Self = Self(1 << 5);
    pub const COLLAPSED: Self = Self(1 << 6);
    pub const EDITABLE: Self = Self(1 << 7);
    pub const OFFSCREEN: Self = Self(1 << 8);
    pub const PASSWORD: Self = Self(1 << 9);

    const NAMES: [(Self, &'static str); 10] = [
        (Self::FOCUSED, "focused"),
        (Self::DISABLED, "disabled"),
        (Self::SELECTED, "selected"),
        (Self::CHECKED, "checked"),
        (Self::MIXED, "mixed"),
        (Self::EXPANDED, "expanded"),
        (Self::COLLAPSED, "collapsed"),
        (Self::EDITABLE, "editable"),
        (Self::OFFSCREEN, "offscreen"),
        (Self::PASSWORD, "password"),
    ];

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn set(&mut self, other: Self, enabled: bool) {
        if enabled {
            self.insert(other);
        }
    }

    pub fn names(self) -> impl Iterator<Item = &'static str> {
        Self::NAMES
            .into_iter()
            .filter(move |(flag, _)| self.contains(*flag))
            .map(|(_, name)| name)
    }
}
