namespace SunCode.Desktop.Controls;

public sealed record SCProviderMetadata(string Title, string Description, string ApiKeyPlaceholder);

public static class SCProviderCatalog
{
    private static readonly IReadOnlyDictionary<string, SCProviderMetadata> Providers =
        new Dictionary<string, SCProviderMetadata>(StringComparer.Ordinal)
        {
            ["deepseek"] = new("DeepSeek", "Configure the URL and credential used by the local DeepSeek provider.", "Paste DeepSeek API key"),
            ["zhipu"] = new("Zhipu GLM", "Configure the URL and credential used by the local Zhipu GLM provider.", "Paste Zhipu API key"),
            ["openai"] = new("OpenAI", "Configure the URL and credential used by the local OpenAI provider.", "Paste OpenAI API key"),
            ["kimi"] = new("Kimi", "Configure the URL and credential used by the local Kimi provider.", "Paste Kimi API key"),
            ["claude"] = new("Claude", "Configure the URL and credential used by the local Claude provider.", "Paste Anthropic API key"),
            ["gemini"] = new("Gemini", "Configure the URL and credential used by the local Gemini provider.", "Paste Gemini API key")
        };

    public static bool TryGet(string providerId, out SCProviderMetadata metadata) =>
        Providers.TryGetValue(providerId, out metadata!);

    public static SCProviderMetadata GetOrDefault(string providerId) =>
        TryGet(providerId, out var metadata)
            ? metadata
            : new SCProviderMetadata(providerId, "Configure the URL and credential used by this local provider.", "Paste API key");
}
