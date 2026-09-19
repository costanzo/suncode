use crate::{
    ComputerAction, ComputerBackend, ComputerFrame, KeyChord, KeyModifier, KeyState, MouseButton,
    MouseButtonState, PixelPoint,
};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::thread;
use std::time::{Duration, Instant};

pub const HALT_MESSAGE: &str = "Not executed: an earlier computer action in this turn failed.";
const WAIT_SLICE: Duration = Duration::from_millis(50);

pub type ComputerResult<T> = Result<T, ComputerError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComputerError {
    InvalidAction(&'static str),
    InvalidFrame(&'static str),
    FrameRequired,
    FrameRetired,
    CoordinateOutOfBounds,
    Cancelled,
    Backend(String),
}

impl Display for ComputerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAction(message) => write!(formatter, "invalid computer action: {message}"),
            Self::InvalidFrame(message) => write!(formatter, "invalid computer frame: {message}"),
            Self::FrameRequired => write!(formatter, "a current screenshot is required"),
            Self::FrameRetired => write!(formatter, "the screenshot coordinate frame changed"),
            Self::CoordinateOutOfBounds => {
                write!(formatter, "coordinate is outside the screenshot")
            }
            Self::Cancelled => write!(formatter, "computer action was cancelled"),
            Self::Backend(message) => write!(formatter, "computer backend failed: {message}"),
        }
    }
}

impl Error for ComputerError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionOutcome {
    Text(String),
    Image(ComputerFrame),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchItem {
    Succeeded(ActionOutcome),
    Failed(String),
    Skipped(String),
}

pub struct ComputerExecutor<B: ComputerBackend> {
    backend: B,
    last_frame: Option<ComputerFrame>,
    held_keys: Vec<String>,
    left_button_held: bool,
}

impl<B: ComputerBackend> ComputerExecutor<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            last_frame: None,
            held_keys: Vec::new(),
            left_button_held: false,
        }
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    pub fn execute_batch(
        &mut self,
        actions: &[ComputerAction],
        cancelled: &dyn Fn() -> bool,
    ) -> Vec<BatchItem> {
        let mut failed = false;
        actions
            .iter()
            .map(|action| {
                if failed {
                    return BatchItem::Skipped(HALT_MESSAGE.into());
                }
                match self.execute(action, cancelled) {
                    Ok(result) => BatchItem::Succeeded(result),
                    Err(error) => {
                        failed = true;
                        let _ = self.release_all();
                        BatchItem::Failed(error.to_string())
                    }
                }
            })
            .collect()
    }

    pub fn execute(
        &mut self,
        action: &ComputerAction,
        cancelled: &dyn Fn() -> bool,
    ) -> ComputerResult<ActionOutcome> {
        action.validate()?;
        self.check_cancelled(cancelled)?;
        match action {
            ComputerAction::Screenshot => {
                let frame = self.backend.capture_primary()?;
                self.last_frame = Some(frame.clone());
                Ok(ActionOutcome::Image(frame))
            }
            ComputerAction::Zoom { region } => {
                let frame = self.current_frame()?.crop(*region)?;
                Ok(ActionOutcome::Image(frame))
            }
            ComputerAction::LeftClick {
                coordinate,
                modifiers,
            } => self.click(MouseButton::Left, *coordinate, modifiers, 1, cancelled),
            ComputerAction::RightClick {
                coordinate,
                modifiers,
            } => self.click(MouseButton::Right, *coordinate, modifiers, 1, cancelled),
            ComputerAction::MiddleClick {
                coordinate,
                modifiers,
            } => self.click(MouseButton::Middle, *coordinate, modifiers, 1, cancelled),
            ComputerAction::DoubleClick {
                coordinate,
                modifiers,
            } => self.click(MouseButton::Left, *coordinate, modifiers, 2, cancelled),
            ComputerAction::TripleClick {
                coordinate,
                modifiers,
            } => self.click(MouseButton::Left, *coordinate, modifiers, 3, cancelled),
            ComputerAction::LeftClickDrag {
                start_coordinate,
                coordinate,
                modifiers,
            } => {
                let start = self.resolve_point(*start_coordinate)?;
                let end = self.resolve_point(*coordinate)?;
                self.with_modifiers(modifiers, cancelled, |executor| {
                    executor.backend.move_pointer(start.0, start.1)?;
                    executor
                        .backend
                        .mouse_button(MouseButton::Left, MouseButtonState::Press)?;
                    executor.left_button_held = true;
                    executor.check_cancelled(cancelled)?;
                    executor.backend.move_pointer(end.0, end.1)?;
                    executor
                        .backend
                        .mouse_button(MouseButton::Left, MouseButtonState::Release)?;
                    executor.left_button_held = false;
                    Ok(())
                })?;
                Ok(ok())
            }
            ComputerAction::MouseMove { coordinate } => {
                let point = self.resolve_point(*coordinate)?;
                self.backend.move_pointer(point.0, point.1)?;
                Ok(ok())
            }
            ComputerAction::LeftMouseDown => {
                self.backend
                    .mouse_button(MouseButton::Left, MouseButtonState::Press)?;
                self.left_button_held = true;
                Ok(ok())
            }
            ComputerAction::LeftMouseUp => {
                self.backend
                    .mouse_button(MouseButton::Left, MouseButtonState::Release)?;
                self.left_button_held = false;
                Ok(ok())
            }
            ComputerAction::CursorPosition => {
                let (x, y) = self.backend.pointer_position()?;
                Ok(ActionOutcome::Text(format!("[{x}, {y}]")))
            }
            ComputerAction::Scroll {
                direction,
                scroll_amount,
                coordinate,
                modifiers,
            } => {
                if let Some(coordinate) = coordinate {
                    let point = self.resolve_point(*coordinate)?;
                    self.backend.move_pointer(point.0, point.1)?;
                }
                self.with_modifiers(modifiers, cancelled, |executor| {
                    executor.backend.scroll(*direction, *scroll_amount)
                })?;
                Ok(ok())
            }
            ComputerAction::Type { text } => {
                self.backend.type_text(text)?;
                Ok(ok())
            }
            ComputerAction::Key { text, repeat } => {
                let chord = KeyChord::parse(text)?;
                for _ in 0..*repeat {
                    self.check_cancelled(cancelled)?;
                    self.execute_chord(&chord, KeyState::Click, cancelled)?;
                }
                Ok(ok())
            }
            ComputerAction::HoldKey {
                text,
                duration_seconds,
            } => {
                let chord = KeyChord::parse(text)?;
                self.press_modifiers(&chord.modifiers)?;
                let result = self.backend.key(&chord.key, KeyState::Press).and_then(|_| {
                    self.held_keys.push(chord.key.clone());
                    self.wait(Duration::from_secs_f64(*duration_seconds), cancelled)
                });
                let key_release = self.backend.key(&chord.key, KeyState::Release);
                self.remove_held(&chord.key);
                let modifier_release = self.release_modifiers(&chord.modifiers);
                result?;
                key_release?;
                modifier_release?;
                Ok(ok())
            }
            ComputerAction::Wait { duration_seconds } => {
                self.wait(Duration::from_secs_f64(*duration_seconds), cancelled)?;
                Ok(ok())
            }
        }
    }

    pub fn release_all(&mut self) -> ComputerResult<()> {
        let mut first_error = None;
        if self.left_button_held {
            if let Err(error) = self
                .backend
                .mouse_button(MouseButton::Left, MouseButtonState::Release)
            {
                first_error = Some(error);
            }
            self.left_button_held = false;
        }
        for key in self.held_keys.drain(..).rev() {
            if let Err(error) = self.backend.key(&key, KeyState::Release) {
                first_error.get_or_insert(error);
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    fn click(
        &mut self,
        button: MouseButton,
        coordinate: Option<PixelPoint>,
        modifiers: &[KeyModifier],
        count: usize,
        cancelled: &dyn Fn() -> bool,
    ) -> ComputerResult<ActionOutcome> {
        if let Some(coordinate) = coordinate {
            let point = self.resolve_point(coordinate)?;
            self.backend.move_pointer(point.0, point.1)?;
        }
        self.with_modifiers(modifiers, cancelled, |executor| {
            for _ in 0..count {
                executor.check_cancelled(cancelled)?;
                executor
                    .backend
                    .mouse_button(button, MouseButtonState::Click)?;
            }
            Ok(())
        })?;
        Ok(ok())
    }

    fn current_frame(&mut self) -> ComputerResult<&ComputerFrame> {
        let current = self.backend.primary_display()?;
        let frame = self
            .last_frame
            .as_ref()
            .ok_or(ComputerError::FrameRequired)?;
        if current != frame.display {
            self.last_frame = None;
            return Err(ComputerError::FrameRetired);
        }
        Ok(self.last_frame.as_ref().expect("frame was checked"))
    }

    fn resolve_point(&mut self, point: PixelPoint) -> ComputerResult<(i32, i32)> {
        self.current_frame()?.input_point(point)
    }

    fn execute_chord(
        &mut self,
        chord: &KeyChord,
        state: KeyState,
        cancelled: &dyn Fn() -> bool,
    ) -> ComputerResult<()> {
        self.with_modifiers(&chord.modifiers, cancelled, |executor| {
            executor.backend.key(&chord.key, state)
        })
    }

    fn with_modifiers<T>(
        &mut self,
        modifiers: &[KeyModifier],
        cancelled: &dyn Fn() -> bool,
        operation: impl FnOnce(&mut Self) -> ComputerResult<T>,
    ) -> ComputerResult<T> {
        self.press_modifiers(modifiers)?;
        let result = self
            .check_cancelled(cancelled)
            .and_then(|_| operation(self));
        let release = self.release_modifiers(modifiers);
        match (result, release) {
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error),
            (Ok(value), Ok(())) => Ok(value),
        }
    }

    fn press_modifiers(&mut self, modifiers: &[KeyModifier]) -> ComputerResult<()> {
        for modifier in modifiers {
            let key = modifier.key_name().to_string();
            if let Err(error) = self.backend.key(&key, KeyState::Press) {
                let _ = self.release_all();
                return Err(error);
            }
            self.held_keys.push(key);
        }
        Ok(())
    }

    fn release_modifiers(&mut self, modifiers: &[KeyModifier]) -> ComputerResult<()> {
        let mut first_error = None;
        for modifier in modifiers.iter().rev() {
            let key = modifier.key_name();
            if let Err(error) = self.backend.key(key, KeyState::Release) {
                first_error.get_or_insert(error);
            }
            self.remove_held(key);
        }
        first_error.map_or(Ok(()), Err)
    }

    fn remove_held(&mut self, key: &str) {
        if let Some(index) = self.held_keys.iter().rposition(|held| held == key) {
            self.held_keys.remove(index);
        }
    }

    fn wait(&self, duration: Duration, cancelled: &dyn Fn() -> bool) -> ComputerResult<()> {
        let deadline = Instant::now() + duration;
        while Instant::now() < deadline {
            self.check_cancelled(cancelled)?;
            thread::sleep(WAIT_SLICE.min(deadline.saturating_duration_since(Instant::now())));
        }
        Ok(())
    }

    fn check_cancelled(&self, cancelled: &dyn Fn() -> bool) -> ComputerResult<()> {
        if cancelled() {
            Err(ComputerError::Cancelled)
        } else {
            Ok(())
        }
    }
}

impl<B: ComputerBackend> Drop for ComputerExecutor<B> {
    fn drop(&mut self) {
        let _ = self.release_all();
    }
}

fn ok() -> ActionOutcome {
    ActionOutcome::Text("OK".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DisplayGeometry, PixelRegion, ScrollDirection};

    #[derive(Default)]
    struct FakeBackend {
        events: Vec<String>,
    }

    impl ComputerBackend for FakeBackend {
        fn primary_display(&mut self) -> ComputerResult<DisplayGeometry> {
            Ok(display())
        }

        fn capture_primary(&mut self) -> ComputerResult<ComputerFrame> {
            ComputerFrame::new(display(), vec![0; 8 * 4 * 4])
        }

        fn move_pointer(&mut self, x: i32, y: i32) -> ComputerResult<()> {
            self.events.push(format!("move:{x}:{y}"));
            Ok(())
        }

        fn pointer_position(&mut self) -> ComputerResult<(i32, i32)> {
            Ok((5, 7))
        }

        fn mouse_button(
            &mut self,
            button: MouseButton,
            state: MouseButtonState,
        ) -> ComputerResult<()> {
            self.events.push(format!("mouse:{button:?}:{state:?}"));
            Ok(())
        }

        fn scroll(&mut self, direction: ScrollDirection, amount: u32) -> ComputerResult<()> {
            self.events.push(format!("scroll:{direction:?}:{amount}"));
            Ok(())
        }

        fn type_text(&mut self, text: &str) -> ComputerResult<()> {
            self.events.push(format!("type:{text}"));
            Ok(())
        }

        fn key(&mut self, key: &str, state: KeyState) -> ComputerResult<()> {
            self.events.push(format!("key:{key}:{state:?}"));
            Ok(())
        }
    }

    fn display() -> DisplayGeometry {
        DisplayGeometry {
            generation: 1,
            input_x: 0,
            input_y: 0,
            input_width: 4,
            input_height: 2,
            pixel_width: 8,
            pixel_height: 4,
        }
    }

    #[test]
    fn screenshot_coordinates_map_to_input_coordinates() {
        let mut executor = ComputerExecutor::new(FakeBackend::default());
        executor
            .execute(&ComputerAction::Screenshot, &|| false)
            .unwrap();
        executor
            .execute(
                &ComputerAction::LeftClick {
                    coordinate: Some(PixelPoint { x: 7, y: 3 }),
                    modifiers: vec![],
                },
                &|| false,
            )
            .unwrap();
        assert_eq!(executor.backend.events[0], "move:3:1");
    }

    #[test]
    fn zoom_crops_without_changing_the_parent_coordinate_frame() {
        let mut executor = ComputerExecutor::new(FakeBackend::default());
        executor
            .execute(&ComputerAction::Screenshot, &|| false)
            .unwrap();
        let result = executor
            .execute(
                &ComputerAction::Zoom {
                    region: PixelRegion {
                        x: 2,
                        y: 0,
                        width: 4,
                        height: 2,
                    },
                },
                &|| false,
            )
            .unwrap();
        let ActionOutcome::Image(frame) = result else {
            panic!("expected image");
        };
        assert_eq!(
            (frame.display.pixel_width, frame.display.pixel_height),
            (4, 2)
        );
        assert_eq!(executor.last_frame.as_ref().unwrap().display, display());
    }

    #[test]
    fn batch_stops_after_first_failure_and_answers_every_action() {
        let mut executor = ComputerExecutor::new(FakeBackend::default());
        let results = executor.execute_batch(
            &[
                ComputerAction::MouseMove {
                    coordinate: PixelPoint { x: 0, y: 0 },
                },
                ComputerAction::Type {
                    text: "later".into(),
                },
            ],
            &|| false,
        );
        assert!(matches!(results[0], BatchItem::Failed(_)));
        assert_eq!(results[1], BatchItem::Skipped(HALT_MESSAGE.into()));
    }

    #[test]
    fn key_chords_press_and_release_modifiers_in_order() {
        let mut executor = ComputerExecutor::new(FakeBackend::default());
        executor
            .execute(
                &ComputerAction::Key {
                    text: "ctrl+shift+s".into(),
                    repeat: 1,
                },
                &|| false,
            )
            .unwrap();
        assert_eq!(
            executor.backend.events,
            [
                "key:Control:Press",
                "key:Shift:Press",
                "key:s:Click",
                "key:Shift:Release",
                "key:Control:Release",
            ]
        );
    }
}
