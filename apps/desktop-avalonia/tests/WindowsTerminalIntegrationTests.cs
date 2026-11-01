using System.Text;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Threading;
using RoyalTerminal.Avalonia.Controls;
using RoyalTerminal.Terminal;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace.Terminal;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class WindowsTerminalIntegrationTests(AvaloniaUiFixture ui)
{
    // Use a split string so echoed input cannot be mistaken for command output.
    private const string Command = "Write-Output ('SunCodeConPty' + 'RoundTrip')\r";
    private const string Marker = "SunCodeConPtyRoundTrip";
    private static readonly string[] Arguments = ["-NoLogo", "-NoProfile", "-NoExit", "-Command",
        "if (Get-Command Set-PSReadLineOption -ErrorAction SilentlyContinue) { Set-PSReadLineOption -HistorySaveStyle SaveNothing }"];

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task WindowsPtyCapturesThePromptAndAcceptsInteractiveInput(bool powerShell7)
    {
        if (!OperatingSystem.IsWindows()) return;
        var path = powerShell7 ? (await PowerShellDiscovery.FindAsync(default))?.Shell?.Path : ShellResolver.SystemPath(false);
        if (path is null) return; // PowerShell 7 is an optional host prerequisite.
        using var pty = new WindowsPty();
        var output = new TerminalOutput();
        pty.DataReceived += (data, length) => output.Append(data.AsSpan(0, length));
        // Do not let a PowerShell 7 parent inject its module versions into 5.1.
        Dictionary<string, string>? environment = powerShell7 ? null : new()
        {
            ["PSModulePath"] = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Windows), "System32", "WindowsPowerShell", "v1.0", "Modules")
        };
        pty.Start(path, workingDirectory: Path.GetTempPath(), environment: environment, arguments: Arguments);
        await output.Prompt.Task.WaitAsync(TimeSpan.FromSeconds(15));
        pty.Resize(100, 25);
        pty.Write(Command);
        await output.Result.Task.WaitAsync(TimeSpan.FromSeconds(10));
        Assert.True(pty.IsRunning);
    }

    [Fact]
    public async Task DesktopTerminalControlReceivesPowerShell7OutputAndSendsInput()
    {
        if (!OperatingSystem.IsWindows()) return;
        var shell = (await PowerShellDiscovery.FindAsync(default))?.Shell;
        if (shell is null) return;
        await ui.Run(async () =>
        {
            var resolved = new ResolvedShell(shell.Path, shell.Family, shell.Version, Arguments);
            using var model = new TerminalSessionViewModel("windows-conpty", Path.GetTempPath()) { ResolveShellAsync = () => Task.FromResult(resolved) };
            var view = new TerminalSessionView { DataContext = model };
            var window = new Window { Width = 800, Height = 300, Content = view };
            var terminal = Assert.IsType<TerminalControl>(view.FindControl<TerminalControl>("Terminal"));
            var output = new TerminalOutput();
            terminal.DataReceived += (_, args) => output.Append(args.Data.Span);
            try
            {
                window.Show(); Dispatcher.UIThread.RunJobs();
                await output.Prompt.Task.WaitAsync(TimeSpan.FromSeconds(15));
                Assert.Equal("running", model.State);
                terminal.SendInput(Command);
                await output.Result.Task.WaitAsync(TimeSpan.FromSeconds(10));
            }
            finally { model.RequestStop(); window.Close(); }
        });
    }

    private sealed class TerminalOutput
    {
        private readonly StringBuilder _text = new();
        public TaskCompletionSource Prompt { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
        public TaskCompletionSource Result { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
        public void Append(ReadOnlySpan<byte> data)
        {
            lock (_text)
            {
                _text.Append(Encoding.UTF8.GetString(data));
                var text = _text.ToString();
                if (text.Contains("PS ", StringComparison.Ordinal)) Prompt.TrySetResult();
                if (text.Contains(Marker, StringComparison.Ordinal)) Result.TrySetResult();
            }
        }
    }
}
