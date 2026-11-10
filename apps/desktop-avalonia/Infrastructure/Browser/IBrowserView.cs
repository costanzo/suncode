using Avalonia.Controls;

namespace SunCode.Desktop.Infrastructure;

internal interface IBrowserView : IDisposable
{
    Control Control { get; }
    string Address { get; set; }
    bool CanGoBack { get; }
    bool CanGoForward { get; }
    bool IsLoading { get; }
    event Action? Initialized;
    event Action? NavigationChanged;
    event Action<Exception>? Failed;
    void GoBack();
    void GoForward();
    void Reload();
}
