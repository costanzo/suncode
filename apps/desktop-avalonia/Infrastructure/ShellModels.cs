using SunCode.Sdk.Models;
namespace SunCode.Desktop.Infrastructure;
public sealed record ShellCheckRequest(string Target, ShellConfiguration? Configuration = null);
public sealed record ShellCheckResult(ShellConfiguration Configuration, string Path, string State, string Message, ResolvedShell? Shell, string? ErrorCode);
public sealed record ShellSettingsResult(ShellCheckResult Agent, ShellCheckResult Terminal, ShellCheckResult? Managed, string? InstallDirectory, string Platform, string Architecture);
public sealed record ShellInstallationStatus(string Stage, bool Running, int Progress, string Message, string? ErrorCode, bool UseForBoth);
