using Avalonia.Controls;
using Avalonia.Interactivity;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Review;

public sealed partial class TurnDiffViewer : UserControl
{
    public TurnDiffViewer()
    {
        InitializeComponent();
        DataContextChanged += (_, _) => AttachReviewer();
    }

    private TurnDiffViewModel? _turnDiff;
    private DiffReviewerWindow? _reviewerWindow;

    private void AttachReviewer()
    {
        if (_turnDiff is not null) _turnDiff.ReviewerRequested -= ShowReviewer;
        _reviewerWindow?.Close();
        _turnDiff = (DataContext as DesktopViewModel)?.TurnDiff;
        if (_turnDiff is not null) _turnDiff.ReviewerRequested += ShowReviewer;
    }

    private void ShowReviewer(DiffReviewerViewModel reviewer)
    {
        if (_reviewerWindow is { } existing)
        {
            if (existing.WindowState == WindowState.Minimized) existing.WindowState = WindowState.Normal;
            existing.Activate();
            return;
        }
        if (TopLevel.GetTopLevel(this) is not Window owner || _turnDiff is not { } turnDiff) return;
        var window = new DiffReviewerWindow(reviewer);
        _reviewerWindow = window;
        window.Closed += (_, _) => { _reviewerWindow = null; turnDiff.ReleaseReviewer(reviewer); };
        window.ShowFor(owner);
    }

    private void OpenDiffReviewer(object? sender, RoutedEventArgs e) => ViewModel.TurnDiff.OpenReviewer();

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private void CloseTurnDiff(object? sender, RoutedEventArgs e) => ViewModel.Layout.TurnDiffVisible = false;
    
}
