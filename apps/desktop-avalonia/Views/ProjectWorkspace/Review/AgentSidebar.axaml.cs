using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Input;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Review;

public sealed partial class AgentSidebar : UserControl
{
    public AgentSidebar()
    {
        InitializeComponent();
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    internal void FocusPendingRequest(bool approval)
    {
        if (approval ? !ViewModel.HasPendingApproval : !ViewModel.HasPendingQuestion) return;
        var card = approval ? PendingApprovalCard : PendingQuestionCard;
        card.Focus();
        card.BringIntoView();
    }

    private async void ApproveOnce(object? sender, RoutedEventArgs e) =>
        await ViewModel.ResolveApprovalAsync("allow_once");

    private async void ApproveForSession(object? sender, RoutedEventArgs e) =>
        await ViewModel.ResolveApprovalAsync("allow_session");

    private async void DenyApproval(object? sender, RoutedEventArgs e) =>
        await ViewModel.ResolveApprovalAsync("deny");

    private void SelectQuestionOption(object? sender, PointerPressedEventArgs e)
    {
        ToggleQuestionOption(sender);
        e.Handled = true;
    }

    private void SelectQuestionOptionOnKey(object? sender, KeyEventArgs e)
    {
        if (e.Key is Key.Space or Key.Enter)
        {
            ToggleQuestionOption(sender);
            e.Handled = true;
        }
    }
    
    private void ToggleQuestionOption(object? sender)
    {
        if ((sender as Control)?.DataContext is QuestionOptionItem option)
            ViewModel.ToggleQuestionOption(option);
    }

    private async void ReplyQuestion(object? sender, RoutedEventArgs e) =>
        await ViewModel.ReplyQuestionAsync();

    private async void RejectQuestion(object? sender, RoutedEventArgs e) =>
        await ViewModel.RejectQuestionAsync();

    private async void RetryTurn(object? sender, RoutedEventArgs e)
    {
        await ViewModel.RetryLastTurnAsync();
    }

    private async void DisableFullControl(object? sender, RoutedEventArgs e) =>
        await ViewModel.DisableFullControlAsync();
    
    private void ToggleTurnChanges(object? sender, RoutedEventArgs e) =>
        ViewModel.TurnChangesExpanded = !ViewModel.TurnChangesExpanded;
    
    private async void OpenTurnChange(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is TurnChangeItem change)
            await ViewModel.OpenTurnDiffAsync(ViewModel.TurnChangesTurnId, change.Path);
    }
}
