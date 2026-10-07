"""Convert SunCode JSONL output into the Harbor ATIF-v1.7 shape."""

from __future__ import annotations

import json
from typing import Any, Iterable


def _text_from_message(message: Any) -> str:
    if not isinstance(message, dict):
        return ""
    content = message.get("content", [])
    if isinstance(content, str):
        return content
    if not isinstance(content, list):
        return ""
    return "\n".join(
        part.get("text", "")
        for part in content
        if isinstance(part, dict) and part.get("type") == "text"
    )


def _tool_call(payload: dict[str, Any]) -> dict[str, Any]:
    return {
        "tool_call_id": payload.get("tool_call_id") or payload.get("call_id") or "unknown",
        "function_name": payload.get("name", "unknown"),
        "arguments": payload.get("arguments", {}),
    }


def _event_lines(output: str) -> Iterable[dict[str, Any]]:
    for line in output.splitlines():
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict) and isinstance(value.get("data"), dict):
            yield value


def convert_jsonl(
    output: str,
    *,
    agent_name: str = "suncode",
    agent_version: str = "unknown",
    model_name: str | None = None,
    session_id: str | None = None,
) -> dict[str, Any]:
    """Build a conservative ATIF trajectory from SunCode event envelopes."""

    steps: list[dict[str, Any]] = []
    pending_tool_steps: dict[str, dict[str, Any]] = {}
    final_metrics: dict[str, Any] = {}
    response_text = ""

    def add_step(source: str, message: str | None = None, **extra: Any) -> dict[str, Any]:
        step: dict[str, Any] = {"step_id": len(steps) + 1, "source": source}
        if message:
            step["message"] = message
        step.update(extra)
        steps.append(step)
        return step

    for envelope in _event_lines(output):
        event_type = envelope.get("type")
        data = envelope["data"]
        session_id = session_id or envelope.get("session_id")

        if event_type == "message.user":
            add_step("user", _text_from_message(data.get("message")))
        elif event_type == "message.assistant":
            message = data.get("message", {})
            calls = [_tool_call(call) for call in message.get("tool_calls", [])]
            step = add_step("agent", _text_from_message(message), **({"tool_calls": calls} if calls else {}))
            for call in calls:
                pending_tool_steps[call["tool_call_id"]] = step
        elif event_type == "tool.requested":
            call = _tool_call(data)
            step = add_step("agent", tool_calls=[call])
            pending_tool_steps[call["tool_call_id"]] = step
        elif event_type == "tool.result":
            call_id = data.get("tool_call_id") or data.get("call_id")
            step = pending_tool_steps.get(call_id)
            if step is None:
                step = add_step("agent")
            observation = step.setdefault("observation", {"results": []})
            observation["results"].append(
                {"source_call_id": call_id or "unknown", "content": data.get("result", {})}
            )
        elif event_type == "run.result":
            response = data.get("response", {})
            message = response.get("message", {})
            response_text = _text_from_message(message) or response_text
            usage = response.get("usage")
            if isinstance(usage, dict):
                final_metrics.update(usage)
        elif event_type == "session.resume.result":
            response = data.get("response", {})
            response_text = _text_from_message(response.get("message", {})) or response_text

    if response_text and not any(step.get("source") == "agent" and step.get("message") == response_text for step in steps):
        add_step("agent", response_text)

    trajectory: dict[str, Any] = {
        "schema_version": "ATIF-v1.7",
        "agent": {"name": agent_name, "version": agent_version, "model_name": model_name or "unknown"},
        "steps": steps,
    }
    if session_id:
        trajectory["session_id"] = session_id
    if final_metrics:
        trajectory["final_metrics"] = final_metrics
    return trajectory


def write_trajectory(output: str, path: str, **metadata: Any) -> None:
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(convert_jsonl(output, **metadata), handle, indent=2)
        handle.write("\n")
