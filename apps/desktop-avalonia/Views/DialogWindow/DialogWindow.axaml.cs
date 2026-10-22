using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Views.DialogWindow;

public sealed partial class DialogWindow : Window
{
    private Action _confirm = static () => { };
    private Action<string>? _submitText;

    // Avalonia's XAML loader requires a public parameterless constructor.
    // Runtime callers use the parameterized overload below.
    public DialogWindow()
    {
        InitializeComponent();
        SetIcon();
    }

    public DialogWindow(
        string title,
        string description,
        string target,
        Action confirm,
        string targetLabel = "SESSION",
        string confirmLabel = "Archive session",
        Size? size = null,
        string? note = null)
    {
        InitializeComponent();
        _confirm = confirm;
        TitleText.Text = title;
        DescriptionText.Text = description;
        TargetText.Text = target;
        TargetLabelText.Text = targetLabel;
        ConfirmButton.Content = confirmLabel;
        NoteText.Text = note;
        NoteText.IsVisible = !string.IsNullOrEmpty(note);
        ApplySize(size);
        SetIcon();
    }

    public DialogWindow(
        string title,
        string description,
        string value,
        string confirmLabel,
        Action<string> submit,
        Size? size = null)
    {
        InitializeComponent();
        _submitText = submit;
        Title = title;
        TitleText.Text = title;
        DescriptionText.Text = description;
        EyebrowText.IsVisible = false;
        TargetBorder.IsVisible = false;
        InputBox.IsVisible = true;
        InputBox.Text = value;
        ConfirmButton.Content = confirmLabel;
        ConfirmButton.Classes.Remove("danger");
        ConfirmButton.Classes.Add("primary");
        ConfirmButton.IsEnabled = !string.IsNullOrEmpty(value);
        Opened += (_, _) => Dispatcher.UIThread.Post(() =>
        {
            InputBox.Focus();
            InputBox.SelectAll();
        }, DispatcherPriority.Input);
        ApplySize(size);
        SetIcon();
    }

    private void ApplySize(Size? size)
    {
        if (size is not { } value) return;
        MinWidth = Math.Min(MinWidth, value.Width);
        MinHeight = Math.Min(MinHeight, value.Height);
        Width = value.Width;
        Height = value.Height;
    }

    private void SetIcon()
    {
        Icon = AppIcon.Window;
    }

    private void CancelClicked(object? sender, RoutedEventArgs e) => Close();

    private void ConfirmClicked(object? sender, RoutedEventArgs e) => Confirm();
    
    
    private void Confirm()
    {
        if (_submitText is null)
        {
            _confirm();
            Close();
            return;
        }

        var text = InputBox.Text?.Trim();
        if (string.IsNullOrEmpty(text)) return;
        Close();
        _submitText(text);
    }
    
    private void InputChanged(object? sender, TextChangedEventArgs e) => ConfirmButton.IsEnabled = !string.IsNullOrEmpty(InputBox.Text?.Trim());
    
    private void InputKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key != Key.Enter || e.KeyModifiers.HasFlag(KeyModifiers.Shift)) return;
        e.Handled = true;
        Confirm();
    }

    protected override void OnKeyDown(KeyEventArgs e)
    {
        base.OnKeyDown(e);
        if (e.Handled || e.Key != Key.Escape) return;
        e.Handled = true;
        Close();
    }
}
