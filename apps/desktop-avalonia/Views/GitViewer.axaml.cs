using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views;

public sealed partial class GitViewer : UserControl
{
    private bool _resizeActive;
    private double _resizeStartY;
    private double _resizeStartHeight;

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

    private void GitResizePressed(object? sender, PointerPressedEventArgs e)
    {
        if (sender is not Control handle || !e.GetCurrentPoint(handle).Properties.IsLeftButtonPressed ||
            TopLevel.GetTopLevel(this) is not Window window) return;
        _resizeActive = true;
        _resizeStartY = e.GetPosition(window).Y;
        _resizeStartHeight = Height;
        e.Pointer.Capture(handle);
        e.Handled = true;
    }

    private void GitResizeMoved(object? sender, PointerEventArgs e)
    {
        if (!_resizeActive || TopLevel.GetTopLevel(this) is not Window window) return;
        var delta = e.GetPosition(window).Y - _resizeStartY;
        Height = Math.Clamp(_resizeStartHeight - delta, 240, Math.Max(240, window.Bounds.Height - 300));
        e.Handled = true;
    }

    private void GitResizeReleased(object? sender, PointerReleasedEventArgs e)
    {
        if (!_resizeActive) return;
        _resizeActive = false;
        e.Pointer.Capture(null);
        e.Handled = true;
    }
}
