namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    internal bool HasAgentSdkForUpdates => _sdk is not null;
    internal bool HasLocalUpdateWork => IsBusy || IsTurnActive || IsSessionLoading || Sessions.Any(session => session.IsTurnActive)
        || Terminal.Sessions.Any(session => session.IsRunning) || QueuedInputs.Count > 0;

    internal async Task<bool> HasAgentUpdateWorkAsync()
    {
        if (HasLocalUpdateWork) return true;
        if (_sdk is null) return false; // SDK repair must not require opening it.
        try
        {
            var projects = await _sdk.GetProjectsAsync().WaitAsync(TimeSpan.FromSeconds(5));
            foreach (var project in projects.Projects)
            {
                var sessions = await _sdk.ListSessionsAsync(project.ProjectId).WaitAsync(TimeSpan.FromSeconds(5));
                if (sessions.SessionStates.Values.Any(state => state is not ("idle" or "approval" or "question" or "failed" or "interrupted"))) return true;
            }
            return false;
        }
        catch (Exception) { return true; } // Unknown active work blocks restart.
    }
}
