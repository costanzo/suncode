using Avalonia.Controls;

namespace SunCode.Desktop.Views.Settings;

// Keeps at most one instance of a tool window open: showing it again
// activates the existing window instead of opening a second one.
internal sealed class SingletonWindow<TWindow> where TWindow : Window
{
    private TWindow? _window;

    public bool IsOpen => _window is not null;

    public void Show(Window owner, Func<TWindow> create, Action? closed = null)
    {
        if (_window is not null)
        {
            _window.Activate();
            return;
        }

        var window = create();
        _window = window;
        window.Closed += (_, _) =>
        {
            _window = null;
            closed?.Invoke();
        };
        window.Show(owner);
    }

    public void Close() => _window?.Close();
}
