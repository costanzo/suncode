using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;

namespace SunCode.Desktop.Controls;

public sealed class SCModal : ContentControl
{
    public static readonly StyledProperty<bool> IsOpenProperty =
        AvaloniaProperty.Register<SCModal, bool>(nameof(IsOpen));

    public static readonly StyledProperty<string?> TitleProperty =
        AvaloniaProperty.Register<SCModal, string?>(nameof(Title));

    public static readonly StyledProperty<string?> DescriptionProperty =
        AvaloniaProperty.Register<SCModal, string?>(nameof(Description));

    /// <summary>
    /// The width of the dialog card rendered inside the full-size modal host.
    /// Keep this separate from the control's own Width so the backdrop can
    /// stretch across the entire parent while the card remains constrained.
    /// </summary>
    public static readonly StyledProperty<double> DialogWidthProperty =
        AvaloniaProperty.Register<SCModal, double>(nameof(DialogWidth), 420);

    public static readonly StyledProperty<string> PrimaryButtonTextProperty =
        AvaloniaProperty.Register<SCModal, string>(nameof(PrimaryButtonText), "Save");

    public static readonly StyledProperty<string> SecondaryButtonTextProperty =
        AvaloniaProperty.Register<SCModal, string>(nameof(SecondaryButtonText), "Cancel");

    public static readonly StyledProperty<bool> PrimaryEnabledProperty =
        AvaloniaProperty.Register<SCModal, bool>(nameof(PrimaryEnabled), true);

    public static readonly StyledProperty<bool> PrimaryIsDangerProperty =
        AvaloniaProperty.Register<SCModal, bool>(nameof(PrimaryIsDanger));

    public static readonly StyledProperty<bool> CloseOnOverlayPressProperty =
        AvaloniaProperty.Register<SCModal, bool>(nameof(CloseOnOverlayPress), true);

    public static readonly StyledProperty<bool> CloseOnEscapeProperty =
        AvaloniaProperty.Register<SCModal, bool>(nameof(CloseOnEscape), true);

    public static readonly StyledProperty<bool> HideCloseProperty =
        AvaloniaProperty.Register<SCModal, bool>(nameof(HideClose));

    public static readonly StyledProperty<bool> HidePrimaryButtonProperty =
        AvaloniaProperty.Register<SCModal, bool>(nameof(HidePrimaryButton));

    public event EventHandler<RoutedEventArgs>? PrimaryAction;
    public event EventHandler<RoutedEventArgs>? SecondaryAction;
    public event EventHandler<RoutedEventArgs>? CloseRequested;

    private Border? _backdrop;
    private Border? _dialog;
    private Grid? _dialogLayout;
    private Grid? _header;
    private Button? _primaryButton;
    private Button? _secondaryButton;
    private Button? _closeButton;
    private Control? _previousFocus;

    public bool IsOpen
    {
        get => GetValue(IsOpenProperty);
        set => SetValue(IsOpenProperty, value);
    }

    public string? Title
    {
        get => GetValue(TitleProperty);
        set => SetValue(TitleProperty, value);
    }

    public string? Description
    {
        get => GetValue(DescriptionProperty);
        set => SetValue(DescriptionProperty, value);
    }

    public double DialogWidth
    {
        get => GetValue(DialogWidthProperty);
        set => SetValue(DialogWidthProperty, value);
    }

    public string PrimaryButtonText
    {
        get => GetValue(PrimaryButtonTextProperty);
        set => SetValue(PrimaryButtonTextProperty, value);
    }

    public string SecondaryButtonText
    {
        get => GetValue(SecondaryButtonTextProperty);
        set => SetValue(SecondaryButtonTextProperty, value);
    }

    public bool PrimaryEnabled
    {
        get => GetValue(PrimaryEnabledProperty);
        set => SetValue(PrimaryEnabledProperty, value);
    }

    public bool PrimaryIsDanger
    {
        get => GetValue(PrimaryIsDangerProperty);
        set => SetValue(PrimaryIsDangerProperty, value);
    }

    public bool CloseOnOverlayPress
    {
        get => GetValue(CloseOnOverlayPressProperty);
        set => SetValue(CloseOnOverlayPressProperty, value);
    }

    public bool CloseOnEscape
    {
        get => GetValue(CloseOnEscapeProperty);
        set => SetValue(CloseOnEscapeProperty, value);
    }

    public bool HideClose
    {
        get => GetValue(HideCloseProperty);
        set => SetValue(HideCloseProperty, value);
    }

    public bool HidePrimaryButton
    {
        get => GetValue(HidePrimaryButtonProperty);
        set => SetValue(HidePrimaryButtonProperty, value);
    }

    protected override void OnApplyTemplate(TemplateAppliedEventArgs e)
    {
        base.OnApplyTemplate(e);

        if (_backdrop is not null)
        {
            _backdrop.PointerPressed -= BackdropPointerPressed;
        }

        if (_primaryButton is not null)
        {
            _primaryButton.Click -= PrimaryButtonClicked;
        }

        if (_secondaryButton is not null)
        {
            _secondaryButton.Click -= SecondaryButtonClicked;
        }

        if (_closeButton is not null)
        {
            _closeButton.Click -= CloseButtonClicked;
        }

        _backdrop = e.NameScope.Find<Border>("PART_Backdrop");
        _dialog = e.NameScope.Find<Border>("PART_Dialog");
        _dialogLayout = e.NameScope.Find<Grid>("PART_DialogLayout");
        _header = e.NameScope.Find<Grid>("PART_Header");
        _primaryButton = e.NameScope.Find<Button>("PART_PrimaryButton");
        _secondaryButton = e.NameScope.Find<Button>("PART_SecondaryButton");
        _closeButton = e.NameScope.Find<Button>("PART_CloseButton");

        UpdateDialogMaxHeight();

        if (_backdrop is not null)
        {
            _backdrop.PointerPressed += BackdropPointerPressed;
        }

        if (_primaryButton is not null)
        {
            _primaryButton.Click += PrimaryButtonClicked;
        }

        if (_secondaryButton is not null)
        {
            _secondaryButton.Click += SecondaryButtonClicked;
        }

        if (_closeButton is not null)
        {
            _closeButton.Click += CloseButtonClicked;
        }

        SyncFocusState();
    }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == BoundsProperty)
        {
            UpdateDialogMaxHeight();
        }
        if (change.Property == IsOpenProperty
            || change.Property == PrimaryIsDangerProperty
            || change.Property == DescriptionProperty
            || change.Property == TitleProperty
            || change.Property == HideCloseProperty
            || change.Property == HidePrimaryButtonProperty)
        {
            SyncFocusState();
        }
    }

    private void UpdateDialogMaxHeight()
    {
        if (_dialog is null || Bounds.Height <= 0) return;

        // Keep the card inside the modal host while retaining a small safety
        // allowance around the design-system 24px breathing room. The extra
        // pixels ensure the one-pixel bottom border is not clipped at the host
        // edge on fractional/native window sizes. Content itself is scrollable
        // in the template when it exceeds the available height.
        var maxHeight = Math.Max(160, Bounds.Height - 56);
        if (Math.Abs(_dialog.MaxHeight - maxHeight) > 0.5)
        {
            _dialog.MaxHeight = maxHeight;
        }
    }

    protected override void OnKeyDown(KeyEventArgs e)
    {
        base.OnKeyDown(e);
        if (!IsOpen) return;

        if (CloseOnEscape && e.Key == Key.Escape)
        {
            e.Handled = true;
            CloseRequested?.Invoke(this, new RoutedEventArgs());
            return;
        }

        if (e.Key == Key.Tab)
        {
            var focusable = EnumerateFocusableControls().ToArray();
            if (focusable.Length == 0) return;

            var current = TopLevel.GetTopLevel(this)?.FocusManager?.GetFocusedElement() as Control;
            var first = focusable[0];
            var last = focusable[^1];
            if (e.KeyModifiers.HasFlag(KeyModifiers.Shift) && ReferenceEquals(current, first))
            {
                e.Handled = true;
                last.Focus();
            }
            else if (!e.KeyModifiers.HasFlag(KeyModifiers.Shift) && ReferenceEquals(current, last))
            {
                e.Handled = true;
                first.Focus();
            }
        }
    }

    private void SyncFocusState()
    {
        IsVisible = IsOpen;
        SyncTemplateState();
        if (!IsOpen)
        {
            RestorePreviousFocus();
            return;
        }

        _previousFocus ??= TopLevel.GetTopLevel(this)?.FocusManager?.GetFocusedElement() as Control;
        Dispatcher.UIThread.Post(FocusFirstControl, DispatcherPriority.Input);
    }

    private IEnumerable<Control> EnumerateFocusableControls()
    {
        if (_dialog is null) yield break;

        foreach (var control in _dialog.GetVisualDescendants().OfType<Control>())
        {
            if (!control.IsEffectivelyVisible || !control.IsEnabled || !control.Focusable)
            {
                continue;
            }

            yield return control;
        }
    }

    private void FocusFirstControl()
    {
        var focusTarget = EnumerateFocusableControls().FirstOrDefault();
        focusTarget?.Focus();
    }

    private void SyncTemplateState()
    {
        var hasHeaderContent = !HideClose
            || !string.IsNullOrWhiteSpace(Title)
            || !string.IsNullOrWhiteSpace(Description);
        if (_header is not null)
        {
            _header.IsVisible = hasHeaderContent;
        }
        if (_dialogLayout is not null)
        {
            _dialogLayout.RowSpacing = hasHeaderContent ? 16 : 0;
        }

        if (_primaryButton is not null)
        {
            _primaryButton.Classes.Set("danger", PrimaryIsDanger);
            _primaryButton.Classes.Set("primary", !PrimaryIsDanger);
            _primaryButton.IsVisible = !HidePrimaryButton;
        }

        if (_closeButton is not null)
        {
            _closeButton.IsVisible = !HideClose;
        }

        if (_dialog?.GetVisualDescendants().OfType<TextBlock>().FirstOrDefault(text => text.Name == "PART_DescriptionText") is { } descriptionText)
        {
            descriptionText.IsVisible = !string.IsNullOrWhiteSpace(Description);
        }
    }

    private void RestorePreviousFocus()
    {
        var previous = _previousFocus;
        _previousFocus = null;
        if (previous is not null)
        {
            Dispatcher.UIThread.Post(() => previous.Focus(), DispatcherPriority.Input);
        }
    }

    private void BackdropPointerPressed(object? sender, PointerPressedEventArgs e)
    {
        if (!CloseOnOverlayPress || !ReferenceEquals(e.Source, sender)) return;
        CloseRequested?.Invoke(this, new RoutedEventArgs());
    }

    private void PrimaryButtonClicked(object? sender, RoutedEventArgs e) =>
        PrimaryAction?.Invoke(this, e);

    private void SecondaryButtonClicked(object? sender, RoutedEventArgs e) =>
        SecondaryAction?.Invoke(this, e);

    private void CloseButtonClicked(object? sender, RoutedEventArgs e) =>
        CloseRequested?.Invoke(this, e);
}
