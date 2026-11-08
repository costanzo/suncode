using System.ComponentModel;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Review;

public sealed partial class QuestionWindow : Window
{
    private readonly DesktopViewModel? _viewModel;
    private readonly PendingQuestionItem? _question;
    private readonly string? _sessionId;
    private Window? _owner;

    public QuestionWindow()
    {
        InitializeComponent();
        Icon = AppIcon.Window;
        ResourcesChanged += (_, _) => RefreshLocalizedText();
        Opened += (_, _) =>
        {
            ConstrainToScreen();
            Dispatcher.UIThread.Post(() => CloseButton.Focus(), DispatcherPriority.Input);
        };
        PositionChanged += (_, _) => ConstrainToScreen();
        AddHandler(KeyDownEvent, WindowKeyDown, RoutingStrategies.Tunnel);
    }

    internal QuestionWindow(DesktopViewModel viewModel, PendingQuestionItem question) : this()
    {
        _viewModel = viewModel;
        _question = question;
        _sessionId = viewModel.SelectedSession?.SessionId;
        DataContext = question;
        viewModel.PropertyChanged += ContextChanged;
    }

    internal Task ShowForAsync(Window owner)
    {
        _owner = owner;
        owner.Closed += OwnerClosed;
        return ShowDialog(owner);
    }

    private void ContextChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (_viewModel is null) return;
        if ((e.PropertyName == nameof(DesktopViewModel.PendingQuestion) && !ReferenceEquals(_viewModel.PendingQuestion, _question))
            || (e.PropertyName == nameof(DesktopViewModel.SelectedSession) && _viewModel.SelectedSession?.SessionId != _sessionId))
            Close();
    }

    private void RefreshLocalizedText()
    {
        _question?.OnPropertyChanged(nameof(PendingQuestionItem.ContextText));
        _question?.OnPropertyChanged(nameof(PendingQuestionItem.AnsweredText));
        _question?.OnPropertyChanged(nameof(PendingQuestionItem.ErrorText));
    }

    private void ConstrainToScreen()
    {
        if (Screens.ScreenFromWindow(this) is not { } screen) return;
        FitToScreen(new Size(screen.WorkingArea.Width / screen.Scaling, screen.WorkingArea.Height / screen.Scaling));
    }

    internal void FitToScreen(Size workingArea)
    {
        var width = Math.Max(1, workingArea.Width - 40);
        var height = Math.Max(1, workingArea.Height - 40);
        MinWidth = Math.Min((double)this.FindResource("QuestionWindowMinWidth")!, width);
        MinHeight = Math.Min((double)this.FindResource("QuestionWindowMinHeight")!, height);
        MaxWidth = width;
        MaxHeight = height;
        Width = Math.Min(Width, width);
        Height = Math.Min(Height, height);
    }

    private void OwnerClosed(object? sender, EventArgs e) => Close();

    protected override void OnClosed(EventArgs e)
    {
        if (_viewModel is not null) _viewModel.PropertyChanged -= ContextChanged;
        if (_owner is not null) _owner.Closed -= OwnerClosed;
        base.OnClosed(e);
    }

    private void WindowKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key != Key.Escape) return;
        e.Handled = true;
        Close();
    }

    private void CloseClicked(object? sender, RoutedEventArgs e) => Close();
    private async void SubmitClicked(object? sender, RoutedEventArgs e)
    {
        if (_viewModel is not null) await _viewModel.ReplyQuestionAsync();
    }

    private async void SkipClicked(object? sender, RoutedEventArgs e)
    {
        if (_viewModel is not null) await _viewModel.RejectQuestionAsync();
    }
}
