use std::io::{self, Write};

use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::{args::OutputMode, error::CliError};

#[derive(Debug)]
pub struct CommandReport {
    pub event_type: &'static str,
    pub data: Value,
    pub text: String,
}

pub fn write_event(mode: OutputMode, event: &suncode_sdk::AgentEvent) -> Result<(), CliError> {
    let data = event.payload.clone().into_value();
    let event_type = event.event_type().as_str();
    match mode {
        OutputMode::Text => {
            match event_type {
                "turn.state" => eprintln!("turn: {}", data["state"].as_str().unwrap_or("unknown")),
                "tool.requested" => {
                    eprintln!("tool: {}", data["name"].as_str().unwrap_or("unknown"))
                }
                "tool.state" => eprintln!(
                    "tool {}: {}",
                    data["name"].as_str().unwrap_or("unknown"),
                    data["state"].as_str().unwrap_or("unknown")
                ),
                "context.compacted" => eprintln!("context compacted"),
                _ => {}
            }
            Ok(())
        }
        OutputMode::Jsonl => {
            let envelope = JsonEnvelope {
                schema_version: 1,
                event_type,
                occurred_at: event.occurred_at.clone(),
                session_id: Some(&event.session_id),
                turn_id: event_turn_id(&data),
                data: &data,
            };
            let stdout = io::stdout();
            let mut writer = stdout.lock();
            serde_json::to_writer(&mut writer, &envelope)
                .map_err(|error| CliError::io(error.to_string()))?;
            writeln!(writer).map_err(CliError::from)
        }
    }
}

fn event_turn_id(data: &Value) -> Option<&str> {
    data.get("turn_id").and_then(Value::as_str)
}

#[derive(Serialize)]
struct JsonEnvelope<'a> {
    schema_version: u8,
    #[serde(rename = "type")]
    event_type: &'a str,
    occurred_at: String,
    session_id: Option<&'a str>,
    turn_id: Option<&'a str>,
    data: &'a Value,
}

pub fn write_report(mode: OutputMode, report: &CommandReport) -> Result<(), CliError> {
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    write_report_to(&mut writer, mode, report)
}

pub fn write_error(mode: OutputMode, error: &CliError) {
    match mode {
        OutputMode::Text => eprintln!("{}: {}", error.code, error.message),
        OutputMode::Jsonl => {
            let data = serde_json::json!({"code": error.code, "message": error.message});
            let envelope = envelope("error", &data);
            match serde_json::to_string(&envelope) {
                Ok(value) => println!("{value}"),
                Err(_) => eprintln!("{}: {}", error.code, error.message),
            }
        }
    }
}

fn write_report_to(
    writer: &mut impl Write,
    mode: OutputMode,
    report: &CommandReport,
) -> Result<(), CliError> {
    match mode {
        OutputMode::Text => writeln!(writer, "{}", report.text)?,
        OutputMode::Jsonl => {
            serde_json::to_writer(&mut *writer, &envelope(report.event_type, &report.data))
                .map_err(|error| CliError::io(error.to_string()))?;
            writeln!(writer)?;
        }
    }
    Ok(())
}

fn envelope<'a>(event_type: &'a str, data: &'a Value) -> JsonEnvelope<'a> {
    JsonEnvelope {
        schema_version: 1,
        event_type,
        occurred_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        session_id: None,
        turn_id: None,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonl_report_uses_the_versioned_envelope() {
        let report = CommandReport {
            event_type: "models.result",
            data: serde_json::json!({"models": []}),
            text: "No models".into(),
        };
        let mut output = Vec::new();
        write_report_to(&mut output, OutputMode::Jsonl, &report).unwrap();
        let value: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["type"], "models.result");
        assert!(value["session_id"].is_null());
        assert!(value["turn_id"].is_null());
    }
}
