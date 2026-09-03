namespace SunCode.Desktop.Controls;

public sealed record SCProviderMetadata(
    string Title,
    string Description,
    string ApiKeyPlaceholder,
    string DefaultEndpoint);

public static class SCProviderCatalog
{
    private static readonly IReadOnlyDictionary<string, SCProviderMetadata> Providers =
        new Dictionary<string, SCProviderMetadata>(StringComparer.Ordinal)
        {
            ["deepseek"] = new("DeepSeek", "Configure the provider URL and credential used by the local agent.", "Paste DeepSeek API key", "https://api.deepseek.com"),
            ["zhipu"] = new("Zhipu GLM", "Configure the provider URL and credential used by the local agent.", "Paste Zhipu API key", "https://open.bigmodel.cn/api/paas/v4"),
            ["openai"] = new("OpenAI", "Configure the provider URL and credential used by the local agent.", "Paste OpenAI API key", "https://api.openai.com/v1"),
            ["kimi"] = new("Kimi", "Configure the provider URL and credential used by the local agent.", "Paste Kimi API key", "https://api.moonshot.ai/v1"),
            ["claude"] = new("Claude", "Configure the provider URL and credential used by the local agent.", "Paste Anthropic API key", "https://api.anthropic.com/v1"),
            ["gemini"] = new("Gemini", "Configure the provider URL and credential used by the local agent.", "Paste Gemini API key", "https://generativelanguage.googleapis.com/v1beta/openai")
        };

    public static bool TryGet(string providerId, out SCProviderMetadata metadata) =>
        Providers.TryGetValue(providerId, out metadata!);

    public static SCProviderMetadata GetOrDefault(string providerId) =>
        TryGet(providerId, out var metadata)
            ? metadata
            : new SCProviderMetadata(providerId, "Configure the provider URL and credential used by the local agent.", "Paste API key", string.Empty);
}
