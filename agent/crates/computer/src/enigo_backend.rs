use crate::{
    BackendRuntimeInfo, ComputerBackend, ComputerError, ComputerFrame, ComputerResult,
    DisplayGeometry, KeyState, MouseButton, MouseButtonState, PermissionState, ScrollDirection,
};
use enigo::{
    Axis, Button, CaptureFrame, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Screen,
    Settings,
};

pub struct EnigoBackend {
    enigo: Enigo,
    generation: u64,
    last_geometry: Option<DisplayGeometry>,
}

impl EnigoBackend {
    pub fn new() -> ComputerResult<Self> {
        let settings = Settings {
            open_prompt_to_get_permissions: false,
            ..Settings::default()
        };
        let enigo = Enigo::new(&settings).map_err(backend_error)?;
        Ok(Self {
            enigo,
            generation: 0,
            last_geometry: None,
        })
    }

    fn geometry_from_capture(&mut self, frame: &CaptureFrame) -> ComputerResult<DisplayGeometry> {
        let candidate = DisplayGeometry {
            generation: 0,
            input_x: frame.input_bounds.x,
            input_y: frame.input_bounds.y,
            input_width: frame.input_bounds.width,
            input_height: frame.input_bounds.height,
            pixel_width: frame.pixel_width,
            pixel_height: frame.pixel_height,
        };
        self.update_generation(candidate)
    }

    fn current_geometry(&mut self) -> ComputerResult<DisplayGeometry> {
        let display = self
            .enigo
            .displays()
            .map_err(backend_error)?
            .into_iter()
            .find(|display| display.primary)
            .ok_or_else(|| ComputerError::Backend("primary display is unavailable".into()))?;
        self.update_generation(DisplayGeometry {
            generation: 0,
            input_x: display.input_bounds.x,
            input_y: display.input_bounds.y,
            input_width: display.input_bounds.width,
            input_height: display.input_bounds.height,
            pixel_width: display.pixel_width,
            pixel_height: display.pixel_height,
        })
    }

    fn update_generation(
        &mut self,
        mut candidate: DisplayGeometry,
    ) -> ComputerResult<DisplayGeometry> {
        let changed = self.last_geometry.is_none_or(|previous| {
            previous.input_x != candidate.input_x
                || previous.input_y != candidate.input_y
                || previous.input_width != candidate.input_width
                || previous.input_height != candidate.input_height
                || previous.pixel_width != candidate.pixel_width
                || previous.pixel_height != candidate.pixel_height
        });
        if changed {
            self.generation = self.generation.saturating_add(1).max(1);
        }
        candidate.generation = self.generation;
        self.last_geometry = Some(candidate);
        Ok(candidate)
    }
}

impl ComputerBackend for EnigoBackend {
    fn runtime_info(&mut self) -> ComputerResult<BackendRuntimeInfo> {
        Ok(BackendRuntimeInfo {
            display: self.current_geometry()?,
            capture_permission: PermissionState::Unknown,
            input_permission: PermissionState::Allowed,
        })
    }

    fn primary_display(&mut self) -> ComputerResult<DisplayGeometry> {
        self.current_geometry()
    }

    fn capture_primary(&mut self) -> ComputerResult<ComputerFrame> {
        let frame = self.enigo.capture_primary().map_err(backend_error)?;
        let geometry = self.geometry_from_capture(&frame)?;
        ComputerFrame::new(geometry, frame.rgba)
    }

    fn move_pointer(&mut self, x: i32, y: i32) -> ComputerResult<()> {
        self.enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(backend_error)
    }

    fn pointer_position(&mut self) -> ComputerResult<(i32, i32)> {
        self.enigo.location().map_err(backend_error)
    }

    fn mouse_button(&mut self, button: MouseButton, state: MouseButtonState) -> ComputerResult<()> {
        self.enigo
            .button(
                match button {
                    MouseButton::Left => Button::Left,
                    MouseButton::Middle => Button::Middle,
                    MouseButton::Right => Button::Right,
                },
                match state {
                    MouseButtonState::Press => Direction::Press,
                    MouseButtonState::Release => Direction::Release,
                    MouseButtonState::Click => Direction::Click,
                },
            )
            .map_err(backend_error)
    }

    fn scroll(&mut self, direction: ScrollDirection, amount: u32) -> ComputerResult<()> {
        let amount = i32::try_from(amount)
            .map_err(|_| ComputerError::InvalidAction("scroll amount is too large"))?;
        let (axis, signed) = match direction {
            ScrollDirection::Up => (Axis::Vertical, -amount),
            ScrollDirection::Down => (Axis::Vertical, amount),
            ScrollDirection::Left => (Axis::Horizontal, -amount),
            ScrollDirection::Right => (Axis::Horizontal, amount),
        };
        self.enigo.scroll(signed, axis).map_err(backend_error)
    }

    fn type_text(&mut self, text: &str) -> ComputerResult<()> {
        self.enigo.text(text).map_err(backend_error)
    }

    fn key(&mut self, key: &str, state: KeyState) -> ComputerResult<()> {
        self.enigo
            .key(parse_key(key)?, key_direction(state))
            .map_err(backend_error)
    }
}

fn parse_key(value: &str) -> ComputerResult<Key> {
    let normalized = value.trim().to_ascii_lowercase();
    let key = match normalized.as_str() {
        "shift" => Key::Shift,
        "control" | "ctrl" => Key::Control,
        "alt" | "option" => Key::Alt,
        "super" | "command" | "cmd" | "win" | "windows" => Key::Meta,
        "return" | "enter" => Key::Return,
        "tab" => Key::Tab,
        "escape" | "esc" => Key::Escape,
        "space" => Key::Space,
        "backspace" => Key::Backspace,
        "delete" => Key::Delete,
        "up" | "arrowup" => Key::UpArrow,
        "down" | "arrowdown" => Key::DownArrow,
        "left" | "arrowleft" => Key::LeftArrow,
        "right" | "arrowright" => Key::RightArrow,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" | "page_up" => Key::PageUp,
        "pagedown" | "page_down" => Key::PageDown,
        "f1" => Key::F1,
        "f2" => Key::F2,
        "f3" => Key::F3,
        "f4" => Key::F4,
        "f5" => Key::F5,
        "f6" => Key::F6,
        "f7" => Key::F7,
        "f8" => Key::F8,
        "f9" => Key::F9,
        "f10" => Key::F10,
        "f11" => Key::F11,
        "f12" => Key::F12,
        _ => {
            let mut characters = value.chars();
            let character = characters
                .next()
                .filter(|_| characters.next().is_none())
                .ok_or(ComputerError::InvalidAction("key is unsupported"))?;
            Key::Unicode(character)
        }
    };
    Ok(key)
}

const fn key_direction(state: KeyState) -> Direction {
    match state {
        KeyState::Press => Direction::Press,
        KeyState::Release => Direction::Release,
        KeyState::Click => Direction::Click,
    }
}

fn backend_error(error: impl std::fmt::Display) -> ComputerError {
    ComputerError::Backend(error.to_string())
}
