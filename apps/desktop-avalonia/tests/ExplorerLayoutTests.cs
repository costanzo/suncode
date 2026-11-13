using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Markup.Xaml.Styling;
using Avalonia.Media;
using Avalonia.Styling;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace.Navigation;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class ExplorerLayoutTests(AvaloniaUiFixture ui)
{
    [Theory]
    [InlineData(280, false, "en-US")]
    [InlineData(280, true, "zh-CN")]
    [InlineData(240, false, "zh-CN")]
    [InlineData(240, true, "en-US")]
    public Task DependencyRowsMatchFolderGeometryAndInteractionBackgrounds(int width, bool dark, string locale) => ui.Run(() =>
    {
        var app = Application.Current!;
        var theme = new FluentTheme();
        var foundation = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/FoundationStyles.axaml") };
        var resources = new ResourceInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ThemeResources.axaml") };
        var language = LocalizationService.CreateDictionary(locale);
        app.Styles.Add(theme); app.Styles.Add(foundation);
        app.Resources.MergedDictionaries.Add(resources); app.Resources.MergedDictionaries.Add(language);
        using var model = new DesktopViewModel();
        model.SetSelectedProjectForTests(new("project", "Example project", "/projects/example"));
        model.Explorer.Dependencies.Add(new("dependency", "electricity-manager-server"));
        model.Explorer.ResetRoots();
        var project = model.Explorer.Roots[0];
        project.Children.Clear();
        project.Children.Add(new("src", "src", "directory") { IsLoaded = true });
        project.IsLoaded = true;
        var dependency = model.Explorer.Roots[1].Children[0];
        dependency.Children.Clear();
        dependency.Children.Add(new("ceps-app", "ceps-app", "directory") { IsLoaded = true });
        dependency.IsLoaded = true;
        var explorer = new ProjectExplorer { DataContext = model };
        var window = new Window { Width = width, Height = 500, Content = explorer, RequestedThemeVariant = dark ? ThemeVariant.Dark : ThemeVariant.Light };
        try
        {
            window.Show(); Flush(window);
            var headers = explorer.GetVisualDescendants().OfType<Grid>().Where(grid => grid.Classes.Contains("explorer-row")).ToArray();
            var folderHeader = headers.Single(grid => ReferenceEquals(grid.DataContext, project.Children[0]));
            var dependencyHeader = headers.Single(grid => ReferenceEquals(grid.DataContext, dependency));
            var dependencyChildHeader = headers.Single(grid => ReferenceEquals(grid.DataContext, dependency.Children[0]));
            var tree = explorer.FindControl<TreeView>("ExplorerTree")!;
            tree.SelectedItem = model.Explorer.Roots[1];
            Flush(window);
            Assert.Equal(Colors.Transparent, Assert.IsAssignableFrom<ISolidColorBrush>(dependencyHeader.Background).Color);
            Assert.Equal(Colors.Transparent, Assert.IsAssignableFrom<ISolidColorBrush>(dependencyChildHeader.Background).Color);
            var action = dependencyHeader.GetVisualDescendants().OfType<Button>().Single();
            Assert.True(action.IsEffectivelyVisible);
            Assert.Equal(folderHeader.Bounds.Height, dependencyHeader.Bounds.Height);
            Assert.Equal(22, dependencyHeader.Bounds.Height);
            Assert.Equal(22, action.Bounds.Height);
            window.MouseMove(action.TranslatePoint(new Point(action.Bounds.Width / 2, action.Bounds.Height / 2), window)!.Value);
            Flush(window);
            Assert.Equal(22, dependencyHeader.Bounds.Height);
            Assert.Equal(Colors.Transparent, Assert.IsAssignableFrom<ISolidColorBrush>(dependencyHeader.Background).Color);
            tree.SelectedItem = dependency;
            action.Focus(NavigationMethod.Tab);
            dependency.IsExpanded = false;
            Flush(window);
            Assert.True(action.IsFocused);
            Assert.Equal(Colors.Transparent, Assert.IsAssignableFrom<ISolidColorBrush>(dependencyHeader.Background).Color);
            Assert.Equal(folderHeader.Bounds.Height, dependencyHeader.Bounds.Height);
            dependency.IsExpanded = true;
            Flush(window);
            Assert.Equal(22, dependencyHeader.Bounds.Height);
            action.Flyout!.ShowAt(action);
            Flush(window);
            Assert.True(action.Flyout.IsOpen);
            action.Flyout.Hide();
        }
        finally
        {
            window.Close();
            app.Resources.MergedDictionaries.Remove(language); app.Resources.MergedDictionaries.Remove(resources);
            app.Styles.Remove(foundation); app.Styles.Remove(theme);
        }
    });

    private static void Flush(Window window)
    {
        Dispatcher.UIThread.RunJobs(); window.UpdateLayout(); Dispatcher.UIThread.RunJobs(); window.UpdateLayout();
    }
}
