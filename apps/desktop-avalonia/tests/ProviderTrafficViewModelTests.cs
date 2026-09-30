using System.Text.Json;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class ProviderTrafficViewModelTests
{
    [Fact]
    public void TracksActiveExchangesUntilTheyFinish()
    {
        var traffic = new ProviderTrafficViewModel();
        try
        {
            traffic.Apply(Event("provider.exchange.started", """{"exchange_id":"exchange-1","provider":"openai","model_id":"gpt-test"}"""));
            traffic.Apply(Event("provider.exchange.progress", """{"exchange_id":"exchange-1","uploaded_bytes":2048,"downloaded_bytes":512}"""));

            Assert.Equal("openai / gpt-test: ↑ 2 KiB   ↓ 512 B", traffic.Details);

            traffic.Apply(Event("provider.exchange.completed", """{"exchange_id":"exchange-1"}"""));
            Assert.Equal("No active LLM requests", traffic.Details);
        }
        finally
        {
            traffic.Clear();
        }
    }

    [Fact]
    public void IgnoresEventsWithoutExchangeIdAndClearResetsRates()
    {
        var traffic = new ProviderTrafficViewModel();

        traffic.Apply(Event("provider.exchange.started", """{"provider":"openai"}"""));
        Assert.Equal("No active LLM requests", traffic.Details);

        traffic.Clear();
        Assert.Equal("↑ 0 B/s   ↓ 0 B/s", traffic.Text);
    }

    private static AgentEvent Event(string eventType, string payload) =>
        JsonSerializer.Deserialize<AgentEvent>(
            $$"""{"session_id":"session-1","occurred_at":"now","event_type":"{{eventType}}","payload":{{payload}}}""",
            new JsonSerializerOptions { PropertyNameCaseInsensitive = true })!;
}
