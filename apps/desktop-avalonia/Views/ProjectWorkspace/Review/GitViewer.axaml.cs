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

    private async void RefreshGit(object? sender, RoutedEventArgs e) => await ViewModel.Git.RefreshAsync();

    private async void GitFileSelected(object? sender, SelectionChangedEventArgs e)
    {
        if (e.AddedItems.OfType<GitFileItem>().FirstOrDefault() is { } file)
            await ViewModel.Git.LoadDiffAsync(file, ViewModel.Git.Scope);
    }

    private async void CopyPatch(object? sender, RoutedEventArgs e)
    {
        if (TopLevel.GetTopLevel(this)?.Clipboard is { } clipboard && !string.IsNullOrWhiteSpace(ViewModel.Git.Patch))
            await clipboard.SetTextAsync(ViewModel.Git.Patch);
    }

    private void GitFilterChanged(object? sender, TextChangedEventArgs e)
    {
        if (sender is TextBox field) ViewModel.Git.SetFilter(field.Text ?? string.Empty);
    }

    private void GitScopeAll(object? sender, RoutedEventArgs e) => ViewModel.Git.SetScope("all");
    private void GitScopeStaged(object? sender, RoutedEventArgs e) => ViewModel.Git.SetScope("staged");
    private void GitScopeUnstaged(object? sender, RoutedEventArgs e) => ViewModel.Git.SetScope("unstaged");
    private void CloseGit(object? sender, RoutedEventArgs e) => ViewModel.Layout.GitVisible = false;
}
