using Avalonia;
using Avalonia.Controls;
using Avalonia.Threading;
using Avalonia.Interactivity;
using Avalonia.Input;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Review;

public sealed partial class AgentSidebar : UserControl
{
    private QuestionWindow? _questionWindow;

    internal bool HasExpandedQuestion => _questionWindow?.IsVisible == true;
    internal QuestionWindow? ExpandedQuestionWindow => _questionWindow;

    public AgentSidebar()
    {
        InitializeComponent();
        DataContextChanged += (_, _) => _questionWindow?.Close();
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

    private async void ExpandQuestion(object? sender, RoutedEventArgs e) => await ExpandQuestionAsync();

    internal async Task ExpandQuestionAsync()
    {
        if (_questionWindow is { IsVisible: true } existing)
        {
            existing.Activate();
            return;
        }
        if (DataContext is not DesktopViewModel viewModel || viewModel.PendingQuestion is not { } question
            || TopLevel.GetTopLevel(this) is not Window owner) return;
        var dialog = new QuestionWindow(viewModel, question);
        _questionWindow = dialog;
        ToolTip.SetIsOpen(ExpandQuestionButton, false);
        ToolTip.SetServiceEnabled(ExpandQuestionButton, false);
        try
        {
            await dialog.ShowForAsync(owner);
        }
        finally
        {
            if (ReferenceEquals(_questionWindow, dialog)) _questionWindow = null;
            ToolTip.SetServiceEnabled(ExpandQuestionButton, true);
            Dispatcher.UIThread.Post(() =>
            {
                if (TopLevel.GetTopLevel(this) is null) return;
                if (ReferenceEquals(viewModel.PendingQuestion, question)) ExpandQuestionButton.Focus();
                else ReviewHeading.Focus();
            }, DispatcherPriority.Input);
        }
    }

    protected override void OnDetachedFromVisualTree(VisualTreeAttachmentEventArgs e)
    {
        _questionWindow?.Close();
        base.OnDetachedFromVisualTree(e);
    }

    private async void ReplyQuestion(object? sender, RoutedEventArgs e) =>
        await ViewModel.ReplyQuestionAsync();

    private async void RejectQuestion(object? sender, RoutedEventArgs e) =>
        await ViewModel.RejectQuestionAsync();

    private async void InspectFailure(object? sender, RoutedEventArgs e) => await ViewModel.OpenFailureDetailsAsync();

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
