use crate::{ComputerError, ComputerResult, PixelPoint, PixelRegion};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

pub const MAX_KEY_REPEAT: u32 = 100;
pub const MAX_ACTION_DURATION: Duration = Duration::from_secs(300);
const MAX_TYPED_CHARACTERS: usize = 100_000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum KeyModifier {
    Shift,
    Control,
    Alt,
    Super,
}

impl KeyModifier {
    pub(crate) const fn key_name(self) -> &'static str {
        match self {
            Self::Shift => "Shift",
            Self::Control => "Control",
            Self::Alt => "Alt",
            Self::Super => "Super",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyChord {
    pub modifiers: Vec<KeyModifier>,
    pub key: String,
}

impl KeyChord {
    pub fn parse(value: &str) -> ComputerResult<Self> {
        let parts = value
            .split('+')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        let Some((key, modifiers)) = parts.split_last() else {
            return Err(ComputerError::InvalidAction("key chord is required"));
        };
        if key.chars().count() > 64 || key.chars().any(char::is_control) {
            return Err(ComputerError::InvalidAction("key chord is invalid"));
        }
        let mut parsed = Vec::new();
        for value in modifiers {
            let modifier = match value.to_ascii_lowercase().as_str() {
                "shift" => KeyModifier::Shift,
                "ctrl" | "control" => KeyModifier::Control,
                "alt" | "option" => KeyModifier::Alt,
                "super" | "command" | "cmd" | "win" | "windows" => KeyModifier::Super,
                _ => return Err(ComputerError::InvalidAction("key modifier is unsupported")),
            };
            if !parsed.contains(&modifier) {
                parsed.push(modifier);
            }
        }
        Ok(Self {
            modifiers: parsed,
            key: (*key).to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ComputerAction {
    Screenshot,
    Zoom {
        region: PixelRegion,
    },
    LeftClick {
        coordinate: Option<PixelPoint>,
        #[serde(default)]
        modifiers: Vec<KeyModifier>,
    },
    RightClick {
        coordinate: Option<PixelPoint>,
        #[serde(default)]
        modifiers: Vec<KeyModifier>,
    },
    MiddleClick {
        coordinate: Option<PixelPoint>,
        #[serde(default)]
        modifiers: Vec<KeyModifier>,
    },
    DoubleClick {
        coordinate: Option<PixelPoint>,
        #[serde(default)]
        modifiers: Vec<KeyModifier>,
    },
    TripleClick {
        coordinate: Option<PixelPoint>,
        #[serde(default)]
        modifiers: Vec<KeyModifier>,
    },
    LeftClickDrag {
        start_coordinate: PixelPoint,
        coordinate: PixelPoint,
        #[serde(default)]
        modifiers: Vec<KeyModifier>,
    },
    MouseMove {
        coordinate: PixelPoint,
    },
    LeftMouseDown,
    LeftMouseUp,
    CursorPosition,
    Scroll {
        direction: ScrollDirection,
        scroll_amount: u32,
        coordinate: Option<PixelPoint>,
        #[serde(default)]
        modifiers: Vec<KeyModifier>,
    },
    Type {
        text: String,
    },
    Key {
        text: String,
        #[serde(default = "default_repeat")]
        repeat: u32,
    },
    HoldKey {
        text: String,
        duration_seconds: f64,
    },
    Wait {
        duration_seconds: f64,
    },
}

const fn default_repeat() -> u32 {
    1
}

impl ComputerAction {
    pub fn from_member(name: &str, input: &Value) -> ComputerResult<Self> {
        let mut object = input
            .as_object()
            .cloned()
            .ok_or(ComputerError::InvalidAction(
                "computer member input must be an object",
            ))?;
        object.insert("type".into(), Value::String(name.to_string()));
        for field in ["coordinate", "start_coordinate"] {
            if let Some(value) = object.get_mut(field) {
                if !value.is_null() {
                    *value = normalize_point(value)?;
                }
            }
        }
        if let Some(region) = object.get_mut("region") {
            *region = normalize_region(region)?;
        }
        if matches!(
            name,
            "left_click"
                | "right_click"
                | "middle_click"
                | "double_click"
                | "triple_click"
                | "left_click_drag"
                | "scroll"
        ) {
            if let Some(text) = object.remove("text") {
                let modifiers = text
                    .as_str()
                    .ok_or(ComputerError::InvalidAction(
                        "computer modifier text must be a string",
                    ))?
                    .split('+')
                    .map(normalize_modifier)
                    .collect::<ComputerResult<Vec<_>>>()?;
                object.insert(
                    "modifiers".into(),
                    serde_json::to_value(modifiers).map_err(|_| {
                        ComputerError::InvalidAction("computer modifiers are invalid")
                    })?,
                );
            }
        }
        if matches!(name, "hold_key" | "wait") {
            if let Some(duration) = object.remove("duration") {
                object.insert("duration_seconds".into(), duration);
            }
        }
        let action: Self = serde_json::from_value(Value::Object(object))
            .map_err(|_| ComputerError::InvalidAction("computer member input is invalid"))?;
        action.validate()?;
        Ok(action)
    }

    pub fn validate(&self) -> ComputerResult<()> {
        match self {
            Self::Zoom { region } if region.width == 0 || region.height == 0 => Err(
                ComputerError::InvalidAction("zoom region must be non-empty"),
            ),
            Self::Type { text } if text.chars().count() > MAX_TYPED_CHARACTERS => Err(
                ComputerError::InvalidAction("typed text exceeds the maximum length"),
            ),
            Self::Key { text, repeat } => {
                KeyChord::parse(text)?;
                if !(1..=MAX_KEY_REPEAT).contains(repeat) {
                    return Err(ComputerError::InvalidAction(
                        "key repeat must be between 1 and 100",
                    ));
                }
                Ok(())
            }
            Self::HoldKey {
                text,
                duration_seconds,
            } => {
                KeyChord::parse(text)?;
                validate_duration(*duration_seconds)
            }
            Self::Wait { duration_seconds } => validate_duration(*duration_seconds),
            Self::Scroll { scroll_amount, .. } if *scroll_amount == 0 => Err(
                ComputerError::InvalidAction("scroll amount must be positive"),
            ),
            Self::LeftClick { modifiers, .. }
            | Self::RightClick { modifiers, .. }
            | Self::MiddleClick { modifiers, .. }
            | Self::DoubleClick { modifiers, .. }
            | Self::TripleClick { modifiers, .. }
            | Self::LeftClickDrag { modifiers, .. }
            | Self::Scroll { modifiers, .. } => validate_modifiers(modifiers),
            _ => Ok(()),
        }
    }
}

fn normalize_point(value: &Value) -> ComputerResult<Value> {
    if value.is_object() {
        return Ok(value.clone());
    }
    let values =
        value
            .as_array()
            .filter(|values| values.len() == 2)
            .ok_or(ComputerError::InvalidAction(
                "coordinate must contain x and y",
            ))?;
    let x = values[0]
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(ComputerError::InvalidAction("coordinate x is invalid"))?;
    let y = values[1]
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(ComputerError::InvalidAction("coordinate y is invalid"))?;
    Ok(serde_json::json!({"x":x,"y":y}))
}

fn normalize_region(value: &Value) -> ComputerResult<Value> {
    if value.is_object() {
        return Ok(value.clone());
    }
    let values =
        value
            .as_array()
            .filter(|values| values.len() == 4)
            .ok_or(ComputerError::InvalidAction(
                "zoom region must contain two corners",
            ))?;
    let coordinate = |index: usize| {
        values[index]
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(ComputerError::InvalidAction(
                "zoom region coordinate is invalid",
            ))
    };
    let x0 = coordinate(0)?;
    let y0 = coordinate(1)?;
    let x1 = coordinate(2)?;
    let y1 = coordinate(3)?;
    let width =
        x1.checked_sub(x0)
            .filter(|width| *width > 0)
            .ok_or(ComputerError::InvalidAction(
                "zoom region corners are invalid",
            ))?;
    let height =
        y1.checked_sub(y0)
            .filter(|height| *height > 0)
            .ok_or(ComputerError::InvalidAction(
                "zoom region corners are invalid",
            ))?;
    Ok(serde_json::json!({"x":x0,"y":y0,"width":width,"height":height}))
}

fn normalize_modifier(value: &str) -> ComputerResult<KeyModifier> {
    match value.trim().to_ascii_lowercase().as_str() {
        "shift" => Ok(KeyModifier::Shift),
        "ctrl" | "control" => Ok(KeyModifier::Control),
        "alt" | "option" => Ok(KeyModifier::Alt),
        "super" | "command" | "cmd" | "win" | "windows" => Ok(KeyModifier::Super),
        _ => Err(ComputerError::InvalidAction(
            "computer modifier is unsupported",
        )),
    }
}

fn validate_duration(seconds: f64) -> ComputerResult<()> {
    if !seconds.is_finite()
        || seconds.is_sign_negative()
        || seconds > MAX_ACTION_DURATION.as_secs_f64()
    {
        return Err(ComputerError::InvalidAction(
            "duration must be between 0 and 300 seconds",
        ));
    }
    Ok(())
}

fn validate_modifiers(modifiers: &[KeyModifier]) -> ComputerResult<()> {
    let mut unique = Vec::new();
    for modifier in modifiers {
        if unique.contains(modifier) {
            return Err(ComputerError::InvalidAction(
                "modifier keys must not contain duplicates",
            ));
        }
        unique.push(*modifier);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_claude_coordinate_region_and_modifier_shapes() {
        assert_eq!(
            ComputerAction::from_member(
                "left_click",
                &serde_json::json!({"coordinate":[10,20],"text":"ctrl+shift"}),
            )
            .unwrap(),
            ComputerAction::LeftClick {
                coordinate: Some(PixelPoint { x: 10, y: 20 }),
                modifiers: vec![KeyModifier::Control, KeyModifier::Shift],
            }
        );
        assert_eq!(
            ComputerAction::from_member("zoom", &serde_json::json!({"region":[2,3,8,11]})).unwrap(),
            ComputerAction::Zoom {
                region: PixelRegion {
                    x: 2,
                    y: 3,
                    width: 6,
                    height: 8,
                },
            }
        );
    }
}
