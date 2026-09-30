using System.Collections.ObjectModel;
using System.ComponentModel;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.ViewModels;

// Project hub surface: the recent-project list and whether projects can be
// opened. It wraps a project-less DesktopViewModel that still owns the SDK
// connection and backs the Settings window opened from the hub.
public sealed class ProjectHubViewModel : ObservableObject, IDisposable
{
    private static readonly string[] ForwardedProperties =
    [
        nameof(DesktopViewModel.HasProjects),
        nameof(DesktopViewModel.CanOpenProjects),
        nameof(DesktopViewModel.StatusText),
    ];

    internal ProjectHubViewModel(DesktopViewModel services)
    {
        Services = services;
        Services.PropertyChanged += ServicesPropertyChanged;
    }

    // Backing window view model for the Settings window and project registration.
    internal DesktopViewModel Services { get; }

    public ObservableCollection<ProjectItem> Projects => Services.Projects;
    public bool HasProjects => Services.HasProjects;
    public bool CanOpenProjects => Services.CanOpenProjects;
    public string StatusText => Services.StatusText;

    public Task InitializeAsync() => Services.InitializeAsync();
    public void UpdateLayoutWidth(double width) => Services.UpdateLayoutWidth(width);

    private void ServicesPropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (e.PropertyName is null or "")
        {
            OnPropertyChanged(string.Empty);
            return;
        }
        if (Array.IndexOf(ForwardedProperties, e.PropertyName) >= 0) OnPropertyChanged(e.PropertyName);
    }

    public void Dispose()
    {
        Services.PropertyChanged -= ServicesPropertyChanged;
        Services.Dispose();
    }
}
