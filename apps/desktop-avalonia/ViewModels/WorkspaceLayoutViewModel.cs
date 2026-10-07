using Avalonia;
using Avalonia.Controls;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.ViewModels;

public enum WorkspaceBottomDrawer
{
    Git,
    ProviderTrace,
    ToolActivity
}

public enum WorkspaceResizeTarget
{
    Navigation,
    Review,
    Preview,
    BottomDrawer
}

// Workspace window layout: user pane and drawer preferences, their persisted
// geometry, and the responsive rules that derive what is effectively shown at
// the current window size. Views bind the effective values; they never decide
// visibility themselves. Persistence is signalled through events so the owning
// window view model keeps sole ownership of the UI state store.
public sealed class WorkspaceLayoutViewModel : ObservableObject
{
    private const double ReviewPaneBreakpoint = 1100;
    private const double NavigationPaneBreakpoint = 860;
    private const double CompactWorkspaceBreakpoint = 620;
    private const double PreviewChatBreakpoint = 1030;
    private const double MinimumWorkspaceHeight = 360;
    // Keep supporting panes usable while allowing them to retreat before the
    // conversation reaches its own minimum width/height.
    internal const double DefaultNavigationPaneWidth = 272;
    internal const double DefaultReviewPaneWidth = 312;
    internal const double DefaultBottomDrawerHeight = 360;
    internal const double MinimumNavigationPaneWidth = DefaultNavigationPaneWidth * 2d / 3d;
    internal const double MinimumReviewPaneWidth = DefaultReviewPaneWidth * 2d / 3d;
    internal const double MinimumBottomDrawerHeight = DefaultBottomDrawerHeight / 2d;

    private readonly IViewModelHost _host;
    private bool _navigationVisible = true;
    private bool _reviewVisible = true;
    private bool _previewVisible;
    private bool _childSessionsVisible;
    private bool _navigationPinned = true;
    private bool _explorerVisible;
    private bool _gitVisible;
    private bool _providerTraceVisible;
    private bool _toolActivityVisible;
    private double _layoutWidth = 1440;
    private double _layoutHeight = 900;
    private double _navigationPaneWidth = DefaultNavigationPaneWidth;
    private double _reviewPaneWidth = DefaultReviewPaneWidth;
    private double _bottomDrawerHeight = DefaultBottomDrawerHeight;
    private double _archivedDrawerHeight = 300;
    private bool _archivedDrawerOpen;

    internal WorkspaceLayoutViewModel(IViewModelHost host) => _host = host;

    // Raised when a persisted region choice (left, right, or bottom) changes.
    internal event Action? RegionStateChanged;
    // Raised when a persisted pane or drawer size changes.
    internal event Action? PanelGeometryChanged;

    public bool NavigationVisible { get => _navigationVisible; set { if (SetProperty(ref _navigationVisible, value)) { NotifyNavigationLayoutChanged(); RegionStateChanged?.Invoke(); } } }
    public bool ExplorerVisible { get => _explorerVisible; set { if (SetProperty(ref _explorerVisible, value)) { NotifyNavigationLayoutChanged(); RegionStateChanged?.Invoke(); } } }
    public bool ReviewVisible { get => _reviewVisible; set { if (SetProperty(ref _reviewVisible, value)) { if (value && _childSessionsVisible) { _childSessionsVisible = false; OnPropertyChanged(nameof(ChildSessionsVisible)); } NotifyReviewLayoutChanged(); RegionStateChanged?.Invoke(); } } }
    public bool ChildSessionsVisible { get => _childSessionsVisible; set { if (SetProperty(ref _childSessionsVisible, value)) { if (value && _reviewVisible) { _reviewVisible = false; OnPropertyChanged(nameof(ReviewVisible)); } NotifyReviewLayoutChanged(); RegionStateChanged?.Invoke(); } } }
    public bool NavigationPinned { get => _navigationPinned; set => SetProperty(ref _navigationPinned, value); }
    public bool GitVisible { get => _gitVisible; set { if (SetProperty(ref _gitVisible, value)) { NotifyDrawerLayoutChanged(nameof(EffectiveGitVisible)); RegionStateChanged?.Invoke(); } } }
    public bool ProviderTraceVisible { get => _providerTraceVisible; set { if (SetProperty(ref _providerTraceVisible, value)) { NotifyDrawerLayoutChanged(nameof(EffectiveProviderTraceVisible)); RegionStateChanged?.Invoke(); } } }
    public bool ToolActivityVisible { get => _toolActivityVisible; set { if (SetProperty(ref _toolActivityVisible, value)) { NotifyDrawerLayoutChanged(nameof(EffectiveToolActivityVisible)); RegionStateChanged?.Invoke(); } } }
    public double NavigationPaneWidth { get => _navigationPaneWidth; set { if (SetProperty(ref _navigationPaneWidth, value)) { NotifyNavigationLayoutChanged(); PanelGeometryChanged?.Invoke(); } } }
    public double ReviewPaneWidth { get => _reviewPaneWidth; set { if (SetProperty(ref _reviewPaneWidth, value)) { NotifyReviewLayoutChanged(); PanelGeometryChanged?.Invoke(); } } }
    public double BottomDrawerHeight { get => _bottomDrawerHeight; set { if (SetProperty(ref _bottomDrawerHeight, value)) { NotifyDrawerLayoutChanged(nameof(EffectiveGitVisible)); OnPropertyChanged(nameof(EffectiveProviderTraceVisible)); OnPropertyChanged(nameof(EffectiveToolActivityVisible)); PanelGeometryChanged?.Invoke(); } } }
    public bool ArchivedDrawerOpen { get => _archivedDrawerOpen; set => SetProperty(ref _archivedDrawerOpen, value); }
    public double ArchivedDrawerHeight { get => _archivedDrawerHeight; private set => SetProperty(ref _archivedDrawerHeight, value); }
    internal void UpdateArchivedDrawerHeight(double sidebarHeight) => ArchivedDrawerHeight = Math.Max(180, sidebarHeight / 2d);

    public bool PreviewVisible
    {
        get => _previewVisible;
        set
        {
            if (!SetProperty(ref _previewVisible, value)) return;
            NotifyNavigationLayoutChanged();
            NotifyReviewLayoutChanged();
            OnPropertyChanged(nameof(EffectivePreviewVisible));
            OnPropertyChanged(nameof(PreviewChatVisible));
            OnPropertyChanged(nameof(ConversationWidth));
            OnPropertyChanged(nameof(PreviewGap));
            OnPropertyChanged(nameof(PreviewWidth));
        }
    }

    public bool SessionSidebarVisible => NavigationVisible && !ExplorerVisible;
    public bool ExplorerSidebarVisible => NavigationVisible && ExplorerVisible;
    public bool EffectiveNavigationVisible => !PreviewVisible && NavigationVisible && _layoutWidth > NavigationPaneBreakpoint && NavigationPaneWidth >= MinimumNavigationPaneWidth;
    public bool EffectiveSessionSidebarVisible => EffectiveNavigationVisible && !ExplorerVisible;
    public bool EffectiveExplorerSidebarVisible => EffectiveNavigationVisible && ExplorerVisible;
    public bool EffectiveReviewVisible => !PreviewVisible && (ReviewVisible || ChildSessionsVisible) && _layoutWidth > ReviewPaneBreakpoint && ReviewPaneWidth >= MinimumReviewPaneWidth;
    public bool EffectiveChildSessionsVisible => ChildSessionsVisible && EffectiveReviewVisible;
    public bool EffectiveReviewInspectorVisible => ReviewVisible && EffectiveReviewVisible;
    public bool EffectiveGitVisible => GitVisible && _layoutWidth > CompactWorkspaceBreakpoint && CanShowBottomDrawer;
    public bool EffectiveProviderTraceVisible => ProviderTraceVisible && _layoutWidth > CompactWorkspaceBreakpoint && CanShowBottomDrawer;
    public bool EffectiveToolActivityVisible => ToolActivityVisible && _layoutWidth > CompactWorkspaceBreakpoint && CanShowBottomDrawer;
    private bool CanShowBottomDrawer => BottomDrawerHeight >= MinimumBottomDrawerHeight;
    // The tallest drawer that still leaves the title bar, gap, minimum
    // workspace, and footer on screen.
    internal double MaxBottomDrawerHeight => Math.Max(
        MinimumBottomDrawerHeight, _layoutHeight - 36d - 4d - MinimumWorkspaceHeight - 20d);
    public double EffectiveBottomDrawerHeight => Math.Min(BottomDrawerHeight, MaxBottomDrawerHeight);
    public bool WorkspaceGuttersVisible => _layoutWidth > CompactWorkspaceBreakpoint;
    public bool WorkspaceStatusDetailsVisible => _layoutWidth > CompactWorkspaceBreakpoint;
    public bool EffectivePreviewVisible => PreviewVisible && _host.SelectedProject is not null;
    public bool PreviewChatVisible => !PreviewVisible || _layoutWidth >= PreviewChatBreakpoint;

    public GridLength WorkspaceGutterWidth => WorkspaceGuttersVisible ? new GridLength(34) : new GridLength(0);
    public GridLength WorkspaceGutterGap => WorkspaceGuttersVisible ? new GridLength(4) : new GridLength(0);
    public GridLength NavigationWidth => EffectiveNavigationVisible ? new GridLength(NavigationPaneWidth) : new GridLength(0);
    public GridLength NavigationGap => EffectiveNavigationVisible ? new GridLength(4) : new GridLength(0);
    public GridLength ReviewWidth => EffectiveReviewVisible ? new GridLength(ReviewPaneWidth) : new GridLength(0);
    public GridLength ReviewGap => EffectiveReviewVisible ? new GridLength(4) : new GridLength(0);
    public GridLength ConversationWidth => PreviewVisible
        ? PreviewChatVisible ? new GridLength(2, GridUnitType.Star) : new GridLength(0)
        : new GridLength(1, GridUnitType.Star);
    public GridLength PreviewGap => EffectivePreviewVisible && PreviewChatVisible ? new GridLength(4) : new GridLength(0);
    public GridLength PreviewWidth => EffectivePreviewVisible ? new GridLength(3, GridUnitType.Star) : new GridLength(0);
    public GridLength BottomDrawerGap => EffectiveGitVisible || EffectiveProviderTraceVisible || EffectiveToolActivityVisible ? new GridLength(4) : new GridLength(0);
    public GridLength GitFileListWidth => new(Math.Min(260, Math.Max(230, (_layoutWidth - 80) * 0.24)));

    public void UpdateLayoutSize(double width, double height)
    {
        _layoutWidth = width;
        _layoutHeight = height;
        NotifyResponsiveLayoutChanged();
        OnPropertyChanged(nameof(GitFileListWidth));
    }

    public void UpdateLayoutWidth(double width) => UpdateLayoutSize(width, _layoutHeight);

    /// <summary>
    /// Left gutter sessions button: hides navigation when sessions are already
    /// shown, otherwise shows navigation on the sessions list.
    /// </summary>
    internal void ToggleSessions()
    {
        RestoreNavigationWidthIfCollapsed();
        if (NavigationVisible && !ExplorerVisible)
        {
            NavigationVisible = false;
            return;
        }
        ExplorerVisible = false;
        NavigationVisible = true;
    }

    /// <summary>
    /// Left gutter explorer button. Returns true when the explorer is now shown.
    /// </summary>
    internal bool ToggleExplorer()
    {
        RestoreNavigationWidthIfCollapsed();
        if (NavigationVisible && ExplorerVisible)
        {
            NavigationVisible = false;
            return false;
        }
        ExplorerVisible = true;
        NavigationVisible = true;
        return true;
    }

    /// <summary>
    /// Right gutter review button. Returns true when an open browser preview
    /// was closed as part of the switch.
    /// </summary>
    internal bool ToggleReview()
    {
        RestoreReviewWidthIfCollapsed();
        return ToggleReviewDestination();
    }

    /// <summary>
    /// Right gutter child-sessions button. Returns true when an open browser
    /// preview was closed as part of the switch.
    /// </summary>
    internal bool ToggleChildSessions()
    {
        RestoreReviewWidthIfCollapsed();
        return ToggleChildSessionsDestination();
    }

    /// <summary>
    /// Activates the review destination from the right gutter. Returns true when
    /// an open browser preview was closed as part of the switch.
    /// </summary>
    internal bool ToggleReviewDestination()
    {
        if (PreviewVisible)
        {
            PreviewVisible = false;
            if (!ReviewVisible) ReviewVisible = true;
            return true;
        }

        ReviewVisible = !ReviewVisible;
        return false;
    }

    /// <summary>
    /// Activates the child-sessions destination from the right gutter. Returns
    /// true when an open browser preview was closed as part of the switch.
    /// </summary>
    internal bool ToggleChildSessionsDestination()
    {
        if (PreviewVisible)
        {
            PreviewVisible = false;
            if (!ChildSessionsVisible) ChildSessionsVisible = true;
            return true;
        }

        ChildSessionsVisible = !ChildSessionsVisible;
        return false;
    }

    /// <summary>
    /// Toggles one bottom drawer. Bottom drawers are mutually exclusive, and a
    /// drawer collapsed below its minimum reopens at the default height.
    /// Returns true when the drawer is now open.
    /// </summary>
    internal bool ToggleBottomDrawer(WorkspaceBottomDrawer drawer)
    {
        var open = !IsBottomDrawerVisible(drawer);
        SetBottomDrawerVisible(drawer, open);
        if (!open) return false;
        if (BottomDrawerHeight < MinimumBottomDrawerHeight) BottomDrawerHeight = DefaultBottomDrawerHeight;
        foreach (var other in Enum.GetValues<WorkspaceBottomDrawer>())
        {
            if (other != drawer) SetBottomDrawerVisible(other, false);
        }
        return true;
    }

    internal void CloseBottomDrawers()
    {
        GitVisible = false;
        ProviderTraceVisible = false;
        ToolActivityVisible = false;
    }

    internal bool IsBottomDrawerVisible(WorkspaceBottomDrawer drawer) => drawer switch
    {
        WorkspaceBottomDrawer.Git => GitVisible,
        WorkspaceBottomDrawer.ProviderTrace => ProviderTraceVisible,
        _ => ToolActivityVisible
    };

    private void SetBottomDrawerVisible(WorkspaceBottomDrawer drawer, bool visible)
    {
        switch (drawer)
        {
            case WorkspaceBottomDrawer.Git: GitVisible = visible; break;
            case WorkspaceBottomDrawer.ProviderTrace: ProviderTraceVisible = visible; break;
            default: ToolActivityVisible = visible; break;
        }
    }

    /// <summary>
    /// Applies a resize drag measured from the size captured when the drag
    /// started. Panes cannot go negative; the bottom drawer cannot push the
    /// central workspace below its minimum height.
    /// </summary>
    internal void ApplyResize(WorkspaceResizeTarget target, double startSize, double deltaX, double deltaY)
    {
        switch (target)
        {
            case WorkspaceResizeTarget.Navigation:
                NavigationPaneWidth = Math.Max(0, startSize + deltaX);
                break;
            case WorkspaceResizeTarget.Review:
            case WorkspaceResizeTarget.Preview:
                ReviewPaneWidth = Math.Max(0, startSize - deltaX);
                break;
            case WorkspaceResizeTarget.BottomDrawer:
                BottomDrawerHeight = Math.Clamp(startSize - deltaY, 0, MaxBottomDrawerHeight);
                break;
        }
    }

    internal double ResizeStartSize(WorkspaceResizeTarget target) => target switch
    {
        WorkspaceResizeTarget.Navigation => NavigationPaneWidth,
        WorkspaceResizeTarget.BottomDrawer => BottomDrawerHeight,
        _ => ReviewPaneWidth
    };

    /// <summary>
    /// Ends a resize drag. A region dragged below its minimum closes; its
    /// size is restored to the default the next time it is toggled open.
    /// </summary>
    internal void CompleteResize(WorkspaceResizeTarget target)
    {
        if (target == WorkspaceResizeTarget.Navigation && NavigationPaneWidth < MinimumNavigationPaneWidth)
            NavigationVisible = false;
        else if (target == WorkspaceResizeTarget.Review && ReviewPaneWidth < MinimumReviewPaneWidth)
            ReviewVisible = false;
        else if (target == WorkspaceResizeTarget.BottomDrawer && BottomDrawerHeight < MinimumBottomDrawerHeight)
            CloseBottomDrawers();
    }

    private void RestoreNavigationWidthIfCollapsed()
    {
        if (NavigationPaneWidth < MinimumNavigationPaneWidth) NavigationPaneWidth = DefaultNavigationPaneWidth;
    }

    private void RestoreReviewWidthIfCollapsed()
    {
        if (ReviewPaneWidth < MinimumReviewPaneWidth) ReviewPaneWidth = DefaultReviewPaneWidth;
    }

    private void NotifyNavigationLayoutChanged()
    {
        OnPropertyChanged(nameof(SessionSidebarVisible));
        OnPropertyChanged(nameof(ExplorerSidebarVisible));
        OnPropertyChanged(nameof(EffectiveNavigationVisible));
        OnPropertyChanged(nameof(EffectiveSessionSidebarVisible));
        OnPropertyChanged(nameof(EffectiveExplorerSidebarVisible));
        OnPropertyChanged(nameof(NavigationWidth));
        OnPropertyChanged(nameof(NavigationGap));
    }

    private void NotifyReviewLayoutChanged()
    {
        OnPropertyChanged(nameof(EffectiveReviewVisible));
        OnPropertyChanged(nameof(EffectiveReviewInspectorVisible));
        OnPropertyChanged(nameof(EffectiveChildSessionsVisible));
        OnPropertyChanged(nameof(ReviewWidth));
        OnPropertyChanged(nameof(ReviewGap));
    }

    private void NotifyResponsiveLayoutChanged()
    {
        NotifyNavigationLayoutChanged();
        NotifyReviewLayoutChanged();
        OnPropertyChanged(nameof(EffectiveGitVisible));
        OnPropertyChanged(nameof(EffectiveProviderTraceVisible));
        OnPropertyChanged(nameof(EffectiveToolActivityVisible));
        OnPropertyChanged(nameof(EffectiveBottomDrawerHeight));
        OnPropertyChanged(nameof(BottomDrawerGap));
        OnPropertyChanged(nameof(WorkspaceGuttersVisible));
        OnPropertyChanged(nameof(WorkspaceGutterWidth));
        OnPropertyChanged(nameof(WorkspaceGutterGap));
        OnPropertyChanged(nameof(WorkspaceStatusDetailsVisible));
        OnPropertyChanged(nameof(PreviewChatVisible));
        OnPropertyChanged(nameof(ConversationWidth));
        OnPropertyChanged(nameof(PreviewGap));
        OnPropertyChanged(nameof(PreviewWidth));
    }

    private void NotifyDrawerLayoutChanged(string effectivePropertyName)
    {
        OnPropertyChanged(effectivePropertyName);
        OnPropertyChanged(nameof(EffectiveBottomDrawerHeight));
        OnPropertyChanged(nameof(BottomDrawerGap));
    }
}
