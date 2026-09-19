use crate::{ComputerFrame, ComputerResult, DisplayGeometry, MouseButton, ScrollDirection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Press,
    Release,
    Click,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButtonState {
    Press,
    Release,
    Click,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    Allowed,
    Denied,
    Unknown,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendRuntimeInfo {
    pub display: DisplayGeometry,
    pub capture_permission: PermissionState,
    pub input_permission: PermissionState,
}

/// Synchronous OS-facing backend owned by one serialized Computer Use actor.
pub trait ComputerBackend: Send {
    fn runtime_info(&mut self) -> ComputerResult<BackendRuntimeInfo> {
        Ok(BackendRuntimeInfo {
            display: self.primary_display()?,
            capture_permission: PermissionState::Unknown,
            input_permission: PermissionState::Unknown,
        })
    }
    fn primary_display(&mut self) -> ComputerResult<DisplayGeometry>;
    fn capture_primary(&mut self) -> ComputerResult<ComputerFrame>;
    fn move_pointer(&mut self, x: i32, y: i32) -> ComputerResult<()>;
    fn pointer_position(&mut self) -> ComputerResult<(i32, i32)>;
    fn mouse_button(&mut self, button: MouseButton, state: MouseButtonState) -> ComputerResult<()>;
    fn scroll(&mut self, direction: ScrollDirection, amount: u32) -> ComputerResult<()>;
    fn type_text(&mut self, text: &str) -> ComputerResult<()>;
    fn key(&mut self, key: &str, state: KeyState) -> ComputerResult<()>;
}

impl<T: ComputerBackend + ?Sized> ComputerBackend for Box<T> {
    fn runtime_info(&mut self) -> ComputerResult<BackendRuntimeInfo> {
        (**self).runtime_info()
    }

    fn primary_display(&mut self) -> ComputerResult<DisplayGeometry> {
        (**self).primary_display()
    }

    fn capture_primary(&mut self) -> ComputerResult<ComputerFrame> {
        (**self).capture_primary()
    }

    fn move_pointer(&mut self, x: i32, y: i32) -> ComputerResult<()> {
        (**self).move_pointer(x, y)
    }

    fn pointer_position(&mut self) -> ComputerResult<(i32, i32)> {
        (**self).pointer_position()
    }

    fn mouse_button(&mut self, button: MouseButton, state: MouseButtonState) -> ComputerResult<()> {
        (**self).mouse_button(button, state)
    }

    fn scroll(&mut self, direction: ScrollDirection, amount: u32) -> ComputerResult<()> {
        (**self).scroll(direction, amount)
    }

    fn type_text(&mut self, text: &str) -> ComputerResult<()> {
        (**self).type_text(text)
    }

    fn key(&mut self, key: &str, state: KeyState) -> ComputerResult<()> {
        (**self).key(key, state)
    }
}
