namespace SunCode.Desktop.ViewModels;

// Confirmation copy shown before a destructive Settings action.
internal sealed record ConfirmationRequest(
    string Title,
    string Message,
    string Target,
    string TargetLabel,
    string ConfirmLabel);

// View services the Settings window provides to its page view models.
internal interface ISettingsDialogs
{
    // Shows a modal confirmation; runs confirmed only when the user accepts.
    void Confirm(ConfirmationRequest request, Action confirmed);

    // Copies text to the clipboard; returns false when no clipboard is available.
    Task<bool> CopyTextAsync(string text);
}
