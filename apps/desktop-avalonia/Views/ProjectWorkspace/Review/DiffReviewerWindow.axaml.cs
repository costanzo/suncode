using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Threading;
using SunCode.Desktop.Controls;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Review;

public sealed partial class DiffReviewerWindow : Window
{
    private bool _syncing;
    private Window? _owner;
    private DiffReviewerViewModel? Model => DataContext as DiffReviewerViewModel;

    public DiffReviewerWindow()
    {
        InitializeComponent();
        Icon = AppIcon.Window;
        BeforeEditor.TextArea.TextView.ScrollOffsetChanged += (_, _) => ScrollChanged(BeforeEditor, AfterEditor);
        AfterEditor.TextArea.TextView.ScrollOffsetChanged += (_, _) => ScrollChanged(AfterEditor, BeforeEditor);
        BeforeEditor.ContextToggled += ToggleContext;
        AfterEditor.ContextToggled += ToggleContext;
        ResourcesChanged += (_, _) => RefreshStrings();
        Opened += (_, _) => FitToScreen();
    }

    internal DiffReviewerWindow(DiffReviewerViewModel model) : this()
    {
        DataContext = model;
        model.CloseRequested += Close;
        model.DocumentChanged += RenderDocument;
        model.ChangeRequested += Navigate;
        RefreshStrings();
    }

    internal void ShowFor(Window owner)
    {
        _owner = owner;
        owner.Closed += OwnerClosed;
        Show(owner);
    }

    private void OwnerClosed(object? sender, EventArgs e) => Close();

    private void RefreshStrings()
    {
        if (Model is not { } model) return;
        Title = $"{model.ProjectName} — {LocalizationService.GetString("LocDiffReviewer", "Diff Reviewer")}";
        model.Notify();
    }

    private void FitToScreen()
    {
        if (Screens.ScreenFromWindow(this) is not { } screen) return;
        var width = Math.Max(1, screen.WorkingArea.Width / screen.Scaling - 40);
        var height = Math.Max(1, screen.WorkingArea.Height / screen.Scaling - 40);
        MinWidth = Math.Min(820, width);
        MinHeight = Math.Min(480, height);
        Width = Math.Min(Width, width);
        Height = Math.Min(Height, height);
    }

    private void BodySizeChanged(object? sender, SizeChangedEventArgs e)
    {
        if (ReviewBody is null) return;
        var column = ReviewBody.ColumnDefinitions[0];
        column.MinWidth = Math.Min(180, Math.Max(0, ReviewBody.Bounds.Width - 4));
        column.MaxWidth = Math.Max(column.MinWidth, Math.Min(360, ReviewBody.Bounds.Width - 564));
        if (column.Width.Value > column.MaxWidth) column.Width = new GridLength(column.MaxWidth);
    }

    private void RenderDocument()
    {
        var top = BeforeEditor.VerticalOffset;
        _syncing = true;
        try
        {
            BeforeEditor.SetReview(Model?.Document, true, Model?.SelectedPath ?? string.Empty);
            AfterEditor.SetReview(Model?.Document, false, Model?.SelectedPath ?? string.Empty);
            BeforeEditor.ScrollToVerticalOffset(top);
            AfterEditor.ScrollToVerticalOffset(top);
        }
        finally { _syncing = false; }
    }

    private void ToggleContext(DiffReviewBlock block) => Model?.ToggleContext(block);

    private void Navigate(int index)
    {
        Dispatcher.UIThread.Post(() =>
        {
            if (Model?.Document is not { ChangeCount: > 0 } document) return;
            _syncing = true;
            BeforeEditor.CurrentChange = AfterEditor.CurrentChange = index;
            var line = document.ChangeLine(Math.Min(index, document.ChangeCount - 1));
            BeforeEditor.ScrollToLine(line);
            AfterEditor.ScrollToLine(line);
            BeforeEditor.RedrawReview();
            AfterEditor.RedrawReview();
            Dispatcher.UIThread.Post(() => _syncing = false, DispatcherPriority.Background);
        }, DispatcherPriority.Loaded);
    }

    private void ScrollChanged(DiffCodePane source, DiffCodePane other)
    {
        if (_syncing || Model?.Document is not { } document) return;
        _syncing = true;
        try
        {
            other.ScrollToVerticalOffset(source.VerticalOffset);
            var marker = source.VerticalOffset + source.Bounds.Height / 3;
            var index = 0;
            var last = document.ChangeCount - 1;
            while (index < last)
            {
                var middle = (index + last + 1) / 2;
                var top = source.TextArea.TextView.GetVisualTopByDocumentLine(document.ChangeLine(middle));
                if (top <= marker) index = middle;
                else last = middle - 1;
            }
            Model.TrackChange(index);
            BeforeEditor.CurrentChange = AfterEditor.CurrentChange = index;
        }
        finally { _syncing = false; }
    }

    private async void RefreshClicked(object? sender, RoutedEventArgs e) { if (Model is { } model) await model.RefreshAsync(); }
    private async void AllClicked(object? sender, RoutedEventArgs e) { if (Model is { } model) await model.SetScopeAsync("all"); }
    private async void StagedClicked(object? sender, RoutedEventArgs e) { if (Model is { } model) await model.SetScopeAsync("staged"); }
    private async void UnstagedClicked(object? sender, RoutedEventArgs e) { if (Model is { } model) await model.SetScopeAsync("unstaged"); }
    private void PreviousClicked(object? sender, RoutedEventArgs e) => Model?.MoveChange(-1);
    private void NextClicked(object? sender, RoutedEventArgs e) => Model?.MoveChange(1);

    protected override void OnClosed(EventArgs e)
    {
        BeforeEditor.DisposeReview();
        AfterEditor.DisposeReview();
        if (_owner is not null) _owner.Closed -= OwnerClosed;
        if (Model is { } model)
        {
            model.CloseRequested -= Close;
            model.DocumentChanged -= RenderDocument;
            model.ChangeRequested -= Navigate;
            model.Dispose();
        }
        base.OnClosed(e);
    }
}
