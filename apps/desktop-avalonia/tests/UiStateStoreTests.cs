using System.Text.Json;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Tests;

public sealed class UiStateStoreTests
{
    [Fact]
    public void PersistsAndReloadsProjectAndSettingsState()
    {
        var directory = Path.Combine(Path.GetTempPath(), "suncode-ui-state-tests", Guid.NewGuid().ToString("N"));
        var path = Path.Combine(directory, "ui-state.json");
        try
        {
            using (var store = new UiStateStore(path))
            {
                store.UpdateSettings(settings =>
                {
                    settings.Page = "appearance";
                    settings.ProviderId = "openai";
                    settings.ProvidersExpanded = false;
                });
                store.UpdateProject("project-1", project =>
                {
                    project.LeftRegion = "explorer";
                    project.RightRegion = "closed";
                    project.BottomDrawer = "toolActivity";
                    project.WindowX = 12;
                    project.WindowWidth = 1200;
                    project.RecentContent.Add(new UiRecentContentState { Kind = "file", Path = "src/Main.cs" });
                });
                store.Flush();
            }

            using var reloaded = new UiStateStore(path);
            Assert.Equal("appearance", reloaded.Settings.Page);
            Assert.Equal("openai", reloaded.Settings.ProviderId);
            Assert.False(reloaded.Settings.ProvidersExpanded);
            var project = reloaded.Project("project-1");
            Assert.Equal("explorer", project.LeftRegion);
            Assert.Equal("closed", project.RightRegion);
            Assert.Equal("toolActivity", project.BottomDrawer);
            Assert.Equal(1200, project.WindowWidth);
            Assert.Single(project.RecentContent);
        }
        finally
        {
            if (Directory.Exists(directory)) Directory.Delete(directory, true);
        }
    }

    [Fact]
    public void CorruptStateFallsBackAndPreservesBackup()
    {
        var directory = Path.Combine(Path.GetTempPath(), "suncode-ui-state-tests", Guid.NewGuid().ToString("N"));
        var path = Path.Combine(directory, "ui-state.json");
        Directory.CreateDirectory(directory);
        File.WriteAllText(path, "{not-json");
        try
        {
            using var store = new UiStateStore(path);
            Assert.Equal("defaults", store.Settings.Page);
            Assert.Contains(Directory.GetFiles(directory), value => value.Contains(".corrupt-", StringComparison.Ordinal));
        }
        finally
        {
            if (Directory.Exists(directory)) Directory.Delete(directory, true);
        }
    }

    [Fact]
    public void FutureVersionIsReadableButNeverOverwritten()
    {
        var directory = Path.Combine(Path.GetTempPath(), "suncode-ui-state-tests", Guid.NewGuid().ToString("N"));
        var path = Path.Combine(directory, "ui-state.json");
        Directory.CreateDirectory(directory);
        File.WriteAllText(path, JsonSerializer.Serialize(new { schemaVersion = 99, settings = new { page = "future" } }));
        try
        {
            using (var store = new UiStateStore(path))
            {
                store.UpdateSettings(settings => settings.Page = "defaults");
                store.Flush();
            }
            Assert.Contains("\"schemaVersion\":99", File.ReadAllText(path));
            Assert.Contains("\"page\":\"future\"", File.ReadAllText(path));
        }
        finally
        {
            if (Directory.Exists(directory)) Directory.Delete(directory, true);
        }
    }
}
