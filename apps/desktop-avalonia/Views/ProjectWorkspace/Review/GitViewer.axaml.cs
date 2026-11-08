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
        DataContextChanged += (_, _) => AttachReviewer();
        SizeChanged += (_, _) =>
        {
            GitScopeGroup.IsVisible = GitScopeDivider.IsVisible = Bounds.Width >= 980;
            GitHeaderBranch.Width = Math.Clamp(Bounds.Width - 465, 48, 190);
        };
    }

    private GitReviewViewModel? _git;
    private DiffReviewerWindow? _reviewerWindow;

    private void AttachReviewer()
    {
        if (_git is not null) _git.ReviewerRequested -= ShowReviewer;
        _reviewerWindow?.Close();
        _git = (DataContext as DesktopViewModel)?.Git;
        if (_git is not null) _git.ReviewerRequested += ShowReviewer;
    }

    private void ShowReviewer(DiffReviewerViewModel reviewer)
    {
        if (_reviewerWindow is { } existing)
        {
            if (existing.WindowState == WindowState.Minimized) existing.WindowState = WindowState.Normal;
            existing.Activate();
            return;
        }
        if (TopLevel.GetTopLevel(this) is not Window owner || _git is not { } git) return;
        var window = new DiffReviewerWindow(reviewer);
        _reviewerWindow = window;
        window.Closed += (_, _) => { _reviewerWindow = null; git.ReleaseReviewer(reviewer); };
        window.ShowFor(owner);
    }

    private void OpenDiffReviewer(object? sender, RoutedEventArgs e) => ViewModel.Git.OpenReviewer();

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
