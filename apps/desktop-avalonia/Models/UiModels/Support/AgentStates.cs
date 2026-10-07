namespace SunCode.Desktop.Models;

/// <summary>Turn state vocabulary shared by projection, models, and views.</summary>
public static class TurnStates
{
    public static bool IsTerminal(string state) =>
        state is "completed" or "failed" or "cancelled" or "interrupted";

    public static bool IsUnsuccessfulTerminal(string state) =>
        state is "failed" or "cancelled" or "interrupted";
}

/// <summary>Tool-call state and display vocabulary.</summary>
public static class ToolStates
{
    public static bool IsSucceeded(string state) => state == "succeeded";

    public static bool IsFailed(string state) =>
        state is "failed" or "denied" or "timed_out" or "unknown_completion";

    public static bool IsActive(string state) => !IsSucceeded(state) && !IsFailed(state);

    public static string DisplayText(string state) => state switch
    {
        "requested" or "validating" or "policy_check" or "authorized" => "Preparing",
        "executing" => "Running",
        "awaiting_approval" => "Waiting for approval",
        "awaiting_question" => "Waiting for an answer",
        "succeeded" => "Completed",
        "denied" => "Denied",
        "failed" => "Failed",
        "timed_out" => "Timed out",
        "unknown_completion" or "reconciling" => "Checking result",
        _ => state
    };

    public static string ErrorText(string errorCode) => errorCode switch
    {
        "invalid_arguments" => "The operation arguments were invalid.",
        "authorization_denied" => "The operation was not authorized.",
        "scope_denied" => "The operation was outside the project scope.",
        "process_executable_not_found" => "The executable could not be found.",
        "process_start_failed" => "The process could not be started.",
        "webfetch_failed" => "The web request could not be completed.",
        _ => errorCode.Replace('_', ' ')
    };

    public static string Summary(string toolName) => toolName switch
    {
        "bash" => "Run shell command",
        "webfetch" => "Fetch web content",
        "read" => "Read file",
        "glob" => "Find files",
        "grep" => "Search files",
        "question" => "Ask a question",
        "todowrite" => "Update turn todos",
        "write" => "Write file",
        "edit" => "Edit file",
        _ => string.IsNullOrWhiteSpace(toolName) ? "Run operation" : toolName
    };
}
