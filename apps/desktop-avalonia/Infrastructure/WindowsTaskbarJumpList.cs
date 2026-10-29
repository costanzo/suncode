using System.Runtime.Versioning;
using SunCode.Desktop.Models;
#if WINDOWS
using Windows.UI.StartScreen;
#endif

namespace SunCode.Desktop.Infrastructure;

/// <summary>
/// Publishes recent projects through the Windows SDK Jump List projection.
/// The generated WinRT projection owns the ABI boundary; no hand-written COM
/// vtables are used here.
/// </summary>
[SupportedOSPlatform("windows")]
internal sealed class WindowsTaskbarJumpList
{
    private const int RecentProjectLimit = 10;
    internal static bool IsEnabled => OperatingSystem.IsWindows();

    private readonly Func<IReadOnlyList<ProjectItem>> _projects;

    internal WindowsTaskbarJumpList(Func<IReadOnlyList<ProjectItem>> projects)
    {
        _projects = projects;
    }

    internal static void ConfigureProcessIdentity()
    {
    }

#if WINDOWS
    internal async Task RefreshAsync()
    {
        if (!IsEnabled || !JumpList.IsSupported()) return;
        try
        {
            var list = await JumpList.LoadCurrentAsync();
            list.Items.Clear();
            list.SystemGroupKind = JumpListSystemGroupKind.None;

            var projects = _projects()
                .DistinctBy(item => item.ProjectId)
                .Take(RecentProjectLimit)
                .ToArray();
            var labels = projects
                .GroupBy(ProjectLabel, StringComparer.OrdinalIgnoreCase)
                .ToDictionary(group => group.Key, group => group.Count(), StringComparer.OrdinalIgnoreCase);

            foreach (var project in projects)
            {
                var label = ProjectLabel(project);
                if (labels[label] > 1) label = $"{label} - {project.CanonicalRoot}";
                var item = JumpListItem.CreateWithArguments(
                    $"--suncode-activate {DesktopActivationRequest.Project(project.CanonicalRoot).ToLaunchArgument()}",
                    label);
                item.GroupName = LocalizationService.GetString("LocMenuRecentProjects", "Recent Projects");
                item.Description = project.CanonicalRoot;
                list.Items.Add(item);
            }

            await list.SaveAsync();
        }
        catch (Exception exception)
        {
            DiagnosticLog.Warn("windows.jump-list", $"refresh_failed=true error={exception.Message}");
        }
    }
#else
    internal Task RefreshAsync() => Task.CompletedTask;
#endif

    private static string ProjectLabel(ProjectItem project) =>
        string.IsNullOrWhiteSpace(project.DisplayName) ? project.CanonicalRoot : project.DisplayName;
}
