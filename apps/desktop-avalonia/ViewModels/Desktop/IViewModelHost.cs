using SunCode.Desktop.Models;
using SunCode.Sdk;

namespace SunCode.Desktop.ViewModels;

// Narrow surface that feature view models use to reach the owning window
// view model's SDK connection, project context, and error reporting without
// depending on the whole DesktopViewModel.
internal interface IViewModelHost
{
    AgentSdk? Sdk { get; }
    ProjectItem? SelectedProject { get; }
    bool EnsureSdk();
    Task<bool> EnsureSdkReadyAsync();
    void ReportError(Exception exception);
    void ReportSuccess(string message);
    void ReportPresentationError(string message);
    void SetBusy(bool busy);

    // Latest status or error message reported through this host.
    string StatusText { get; }
}
