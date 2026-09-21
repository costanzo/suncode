namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    private readonly Dictionary<string, string> _sessionComposerDrafts = new(StringComparer.OrdinalIgnoreCase);

    private string? _composerDraftSessionId;
    
    internal void SaveCurrentComposerDraft()
    {
        if (_composerDraftSessionId is { Length: > 0 } sessionId)
        {
            _sessionComposerDrafts[sessionId] = ComposerText ?? string.Empty;
        }
    }
    
    internal void RestoreComposerDraft(string sessionId)
    {
        _composerDraftSessionId = sessionId;
        ComposerText = _sessionComposerDrafts.TryGetValue(sessionId, out var draft) ? draft : string.Empty;
    }
    
    internal void ResetCurrentComposerDraft()
    {
        if (_composerDraftSessionId is { Length: > 0 } sessionId)
        {
            _sessionComposerDrafts[sessionId] = string.Empty;
        }
    }
    
    internal void ForgetComposerDraft(string sessionId)
    {
        _sessionComposerDrafts.Remove(sessionId);
    }
    
    internal void BindComposerDraftToCurrent(string sessionId)
    {
        _composerDraftSessionId = sessionId;
        _sessionComposerDrafts[sessionId] = ComposerText ?? string.Empty;
    }
    
    internal void ClearAllComposerDrafts()
    {
        _sessionComposerDrafts.Clear();
        _composerDraftSessionId = null;
    }
}