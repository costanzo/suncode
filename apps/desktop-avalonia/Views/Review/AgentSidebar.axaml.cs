using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.Projects;

namespace SunCode.Desktop.Views.Review;

public sealed partial class AgentSidebar : UserControl
{
    public AgentSidebar()
    {
        InitializeComponent();
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private async void ApproveOnce(object? sender, RoutedEventArgs e) =>
        await ViewModel.ResolveApprovalAsync("allow_once");

    private async void ApproveForSession(object? sender, RoutedEventArgs e) =>
        await ViewModel.ResolveApprovalAsync("allow_session");

    private async void DenyApproval(object? sender, RoutedEventArgs e) =>
        await ViewModel.ResolveApprovalAsync("deny");

    private void SelectQuestionOption(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is QuestionOptionItem option)
            ViewModel.ToggleQuestionOption(option);
    }

    private async void ReplyQuestion(object? sender, RoutedEventArgs e) =>
        await ViewModel.ReplyQuestionAsync();

    private async void RejectQuestion(object? sender, RoutedEventArgs e) =>
        await ViewModel.RejectQuestionAsync();

    private async void DisableFullControl(object? sender, RoutedEventArgs e) =>
        await ViewModel.DisableFullControlAsync();

    private void RestoreCheckpoint(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is CheckpointItem checkpoint)
            this.FindAncestorOfType<ProjectWorkspace>()?.ShowUndoDialog(checkpoint);
    }

    private void ViewTurnChanges(object? sender, RoutedEventArgs e) =>
        this.FindAncestorOfType<ProjectWorkspace>()?.ToggleGitViewer();
}
