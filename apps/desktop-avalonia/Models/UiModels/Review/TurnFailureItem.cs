using System.Text.Json;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Models;

public sealed record TurnFailureItem(
    string TurnId,
    string Code,
    string Message,
    string ExchangeId = "",
    string Provider = "",
    string ModelId = "")
{
    public bool HasProviderCall => ExchangeId.Length > 0;
    public bool HasProviderModel => Provider.Length > 0 || ModelId.Length > 0;
    public string ProviderModel => string.Join(" · ", new[] { Provider, ModelId }.Where(value => value.Length > 0));
    public string Title => LocalizationService.GetString(
        HasProviderCall ? "LocProviderRequestFailed" : "LocTurnFailed",
        HasProviderCall ? "Provider request failed" : "Turn failed");
    public string Summary => !string.IsNullOrWhiteSpace(Message) ? Message : !string.IsNullOrWhiteSpace(Code) ? Code
        : LocalizationService.GetString("LocFailureReasonUnavailable", "The failure reason is unavailable.");

    internal static TurnFailureItem FromCall(ProviderTraceItem call)
    {
        var code = string.Empty;
        var message = call.ErrorText;
        try
        {
            using var error = JsonDocument.Parse(call.ErrorText);
            if (error.RootElement.ValueKind == JsonValueKind.Object)
            {
                if (error.RootElement.TryGetProperty("code", out var value) && value.ValueKind == JsonValueKind.String)
                    code = value.GetString() ?? string.Empty;
                if (error.RootElement.TryGetProperty("message", out value) && value.ValueKind == JsonValueKind.String)
                    message = value.GetString() ?? string.Empty;
            }
        }
        catch (JsonException) { }
        return new(call.TurnId, code, message, call.ExchangeId, call.Provider, call.ModelId);
    }
}
