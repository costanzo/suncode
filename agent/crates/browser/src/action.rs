//! Typed browser actions shared by the tool contract and the CDP transport.

/// One validated `browser_*` tool call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BrowserAction {
    /// Report the attached page; `browser_open` without a URL and `browser_tabs`.
    State,
    Navigate {
        url: String,
    },
    Snapshot,
    Screenshot {
        full_page: bool,
    },
    Click {
        target: Locator,
    },
    Fill {
        target: Locator,
        value: String,
    },
    /// Key press dispatched to the focused element.
    Press {
        key: String,
    },
    ClosePage,
}

/// Semantic element locator. Arbitrary CSS, XPath, and scripts are not
/// expressible, so the agent targets what users see.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Locator {
    /// Text of a `<label>`, or an `aria-label` attribute.
    Label(String),
    Placeholder(String),
    /// `data-testid` attribute.
    TestId(String),
    /// Exact visible text of a button, link, or form control.
    Text(String),
    /// Explicit `role` attribute plus exact visible text.
    Role {
        role: String,
        name: String,
    },
}
