using Avalonia.Controls;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Review;

public sealed partial class GitViewer : UserControl
{
    public GitViewer()
    {
        InitializeComponent();
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    internal void ClampHeightToWindow()
    {
        if (TopLevel.GetTopLevel(this) is not Window window) return;
        Height = Math.Clamp(Height, 240, Math.Max(240, window.Bounds.Height - 300));
    }

    private async void RefreshGit(object? sender, RoutedEventArgs e) => await ViewModel.RefreshGitAsync();

    private async void GitFileSelected(object? sender, SelectionChangedEventArgs e)
    {
        if (e.AddedItems.OfType<GitFileItem>().FirstOrDefault() is { } file)
            await ViewModel.LoadGitDiffAsync(file, ViewModel.GitScope);
    }

    private async void CopyPatch(object? sender, RoutedEventArgs e)
    {
        if (TopLevel.GetTopLevel(this)?.Clipboard is { } clipboard && !string.IsNullOrWhiteSpace(ViewModel.GitPatch))
            await clipboard.SetTextAsync(ViewModel.GitPatch);
    }

    private void GitFilterChanged(object? sender, TextChangedEventArgs e)
    {
        if (sender is TextBox field) ViewModel.SetGitFilter(field.Text ?? string.Empty);
    }

    private void GitScopeAll(object? sender, RoutedEventArgs e) => ViewModel.SetGitScope("all");
    private void GitScopeStaged(object? sender, RoutedEventArgs e) => ViewModel.SetGitScope("staged");
    private void GitScopeUnstaged(object? sender, RoutedEventArgs e) => ViewModel.SetGitScope("unstaged");
    private void CloseGit(object? sender, RoutedEventArgs e) => ViewModel.GitVisible = false;
}
