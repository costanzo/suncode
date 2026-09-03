using Avalonia.Controls;
using Avalonia.Interactivity;

namespace SunCode.Desktop.Views.DialogWindow;

public sealed partial class DialogWindow : Window
{
    private Action _confirm = static () => { };

    // Avalonia's XAML loader requires a public parameterless constructor.
    // Runtime callers use the parameterized overload below.
    public DialogWindow()
    {
        InitializeComponent();
        SetIcon();
    }

    public DialogWindow(string title, string description, string target, Action confirm)
    {
        InitializeComponent();
        _confirm = confirm;
        TitleText.Text = title;
        DescriptionText.Text = description;
        TargetText.Text = target;
        SetIcon();
    }

    private void SetIcon()
    {
        Icon = new WindowIcon(Avalonia.Platform.AssetLoader.Open(new Uri("avares://SunCode/Assets/logo/suncode-logo-128.png")));
    }

    private void CancelClicked(object? sender, RoutedEventArgs e) => Close();

    private void ConfirmClicked(object? sender, RoutedEventArgs e)
    {
        _confirm();
        Close();
    }
}
