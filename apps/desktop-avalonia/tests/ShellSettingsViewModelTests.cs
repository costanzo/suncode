using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class ShellSettingsViewModelTests
{
    private static ShellCheckResult Result(ShellConfiguration configuration, bool available = true) => new(
        configuration, configuration.Path, available ? "available" : "unavailable", available ? "POSIX sh" : "Executable not found",
        available ? new ResolvedShell(configuration.Path, "sh", "POSIX sh", ["-il"]) : null, available ? null : "shell_executable_not_found");

    [Fact]
    public async Task CustomPathMustPassDetectionBeforeItCanBeSaved()
    {
        var checkedRequest = new TaskCompletionSource<ShellCheckRequest>(TaskCreationOptions.RunContinuationsAsynchronously);
        var detection = new TaskCompletionSource<ShellCheckResult>(TaskCreationOptions.RunContinuationsAsynchronously);
        ShellCheckRequest? saved = null;
        using var editor = new ShellEditorViewModel("agent", request => { checkedRequest.TrySetResult(request); return detection.Task; }, request => { saved = request; return Task.FromResult(Result(request.Configuration!)); });
        editor.Apply(Result(new()), false);
        editor.SelectedOption = editor.Options.Single(option => Equals(option.Value, "custom"));
        Assert.False(editor.SaveCommand.CanExecute(null));
        editor.CustomPath = "/bin/sh";
        var request = await checkedRequest.Task.WaitAsync(TimeSpan.FromSeconds(3));
        Assert.False(editor.SaveCommand.CanExecute(null));
        detection.SetResult(Result(request.Configuration!));
        await WaitUntil(() => editor.SaveCommand.CanExecute(null));
        await ((AsyncRelayCommand)editor.SaveCommand).ExecuteAsync();
        Assert.Equal("agent", saved?.Target);
        Assert.Equal("/bin/sh", saved?.Configuration?.Path);
        Assert.False(editor.IsDirty);
        Assert.False(editor.SaveCommand.CanExecute(null));
    }

    [Fact]
    public async Task LateDetectionCannotValidateANewerPath()
    {
        var oldStarted = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        var newStarted = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        var oldResult = new TaskCompletionSource<ShellCheckResult>(TaskCreationOptions.RunContinuationsAsynchronously);
        var newResult = new TaskCompletionSource<ShellCheckResult>(TaskCreationOptions.RunContinuationsAsynchronously);
        using var editor = new ShellEditorViewModel("terminal", request =>
        {
            if (request.Configuration!.Path == "/bin/sh") { oldStarted.SetResult(); return oldResult.Task; }
            newStarted.SetResult(); return newResult.Task;
        }, _ => throw new InvalidOperationException("A failed path must never be saved"));
        editor.Apply(Result(new()), false);
        editor.SelectedOption = editor.Options.Single(option => Equals(option.Value, "custom"));
        editor.CustomPath = "/bin/sh";
        await oldStarted.Task.WaitAsync(TimeSpan.FromSeconds(3));
        editor.CustomPath = "/missing/bash";
        oldResult.SetResult(Result(new("custom", "/bin/sh")));
        await newStarted.Task.WaitAsync(TimeSpan.FromSeconds(3));
        Assert.False(editor.SaveCommand.CanExecute(null));
        newResult.SetResult(Result(new("custom", "/missing/bash"), false));
        await WaitUntil(() => editor.StatusTone == StatusTone.Danger);
        Assert.True(editor.IsDirty);
        Assert.False(editor.SaveCommand.CanExecute(null));
    }

    [Fact]
    public void ManagedInstallationCanBeSelectedIndependentlyAndUseBothDisablesAfterSelection()
    {
        using var settings = new ShellSettingsViewModel(new FakeViewModelHost());
        var managed = Result(new("managed"));
        settings.ApplySettings(new(managed, Result(new()), managed, "managed-runtime", "windows", "arm64"));
        Assert.True(settings.IsWindows);
        Assert.True(settings.Installed);
        Assert.Equal("Windows ARM64", settings.Architecture);
        Assert.True(settings.UseBothCommand.CanExecute(null));
        settings.ApplySettings(new(managed, managed, managed, "managed-runtime", "windows", "arm64"), true);
        Assert.False(settings.UseBothCommand.CanExecute(null));
    }

    [Fact]
    public void ExistingInstallationReplacesDownloadOfferAndDoesNotOverwriteChoices()
    {
        using var settings = new ShellSettingsViewModel(new FakeViewModelHost());
        Assert.True(settings.IsDiscovering);
        Assert.False(settings.CanInstall);
        var existing = PowerShellDiscoveryTests.Available(Path.Combine(Path.GetTempPath(), "existing", "pwsh.exe"));
        var custom = Result(new("custom", Path.Combine(Path.GetTempPath(), "other", "pwsh.exe")));
        settings.ApplySettings(new(custom, Result(new()), null, "managed-runtime", "windows", "x64", existing));
        Assert.False(settings.IsDiscovering);
        Assert.True(settings.HasExisting);
        Assert.False(settings.CanInstall);
        Assert.False(settings.ShowInstallChoices);
        Assert.False(settings.InstallCommand.CanExecute(null));
        Assert.True(settings.UseExistingCommand.CanExecute(null));
        Assert.Equal(custom.Configuration, settings.Agent.SavedConfiguration);
        Assert.Equal("system", settings.Terminal.SavedConfiguration.Mode);
        Assert.Equal(existing.Shell!.Path, settings.ExistingPath);

        settings.ApplySettings(new(existing, existing, null, "managed-runtime", "windows", "x64", existing), true);
        Assert.False(settings.UseExistingCommand.CanExecute(null));

        settings.ApplySettings(new(Result(new()), Result(new()), null, "managed-runtime", "windows", "x64"), true);
        Assert.False(settings.HasExisting);
        Assert.True(settings.CanInstall);
        Assert.True(settings.ShowInstallChoices);
        Assert.False(settings.UseExistingCommand.CanExecute(null));
    }

    private static async Task WaitUntil(Func<bool> predicate)
    {
        using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(3));
        while (!predicate()) await Task.Delay(10, timeout.Token);
    }
}
