using System.Text.Json;
using SunCode.Sdk;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class SdkTypedModelTests
{
    [Fact]
    public async Task VersionComesFromTheRustAgentCore()
    {
        var version = await AgentSdk.GetVersionAsync();

        Assert.Equal("0.1.0", version.Version);
    }

    private static readonly JsonSerializerOptions Options = new()
    {
        PropertyNameCaseInsensitive = true,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        NumberHandling = System.Text.Json.Serialization.JsonNumberHandling.AllowReadingFromString
    };

    [Fact]
    public void Deserializes_snake_case_usage_and_turn_fields()
    {
        const string json = "{\"session_id\":\"session-1\",\"input_tokens\":12,\"output_tokens\":8,\"total_tokens\":20}";
        var usage = JsonSerializer.Deserialize<SessionUsageResult>(json, Options);

        Assert.NotNull(usage);
        Assert.Equal("session-1", usage.SessionId);
        Assert.Equal((ulong)20, usage.TotalTokens);
    }

    [Fact]
    public void Deserializes_camel_case_project_and_session_models()
    {
        const string json = "{\"projectId\":\"project-1\",\"canonicalRoot\":\"/tmp/project\",\"displayName\":\"Project\",\"createdAt\":\"now\",\"updatedAt\":\"now\",\"lastOpenedAt\":\"now\",\"archivedAt\":null}";
        var project = JsonSerializer.Deserialize<ProjectRecord>(json, Options);

        Assert.NotNull(project);
        Assert.Equal("project-1", project.ProjectId);
        Assert.Equal("Project", project.DisplayName);

        const string sessionJson = "{\"sessionId\":\"session-1\",\"projectId\":\"project-1\",\"title\":\"Chat\",\"modelId\":\"gpt-5.5\",\"reasoningEffort\":\"high\",\"status\":\"active\",\"createdAt\":\"now\",\"updatedAt\":\"now\",\"lastActivityAt\":\"now\",\"archivedAt\":null,\"pinAt\":null}";
        var session = JsonSerializer.Deserialize<SessionRecord>(sessionJson, Options);

        Assert.NotNull(session);
        Assert.Equal("gpt-5.5", session.ModelId);
        Assert.Equal("high", session.ReasoningEffort);
    }

    [Fact]
    public void Deserializes_bounded_project_file_result()
    {
        const string json = "{\"projectId\":\"project-1\",\"dependencyId\":null,\"path\":\"src/main.rs\",\"content\":\"fn main() {}\\n\",\"bytes\":13}";
        var file = JsonSerializer.Deserialize<ProjectFileResult>(json, Options);

        Assert.NotNull(file);
        Assert.Equal("project-1", file.ProjectId);
        Assert.Equal("src/main.rs", file.Path);
        Assert.Equal("fn main() {}\n", file.Content);
        Assert.Equal((ulong)13, file.Bytes);
    }

    [Fact]
    public void Serializes_mcp_transport_with_rust_discriminator_and_camel_case_fields()
    {
        var request = new McpServerWriteRequest(
            "Local MCP",
            new McpStdioTransportRequest(
                "uvx",
                ["mcp-server"],
                "project",
                new McpSecretChanges(
                    new Dictionary<string, string> { ["TOKEN"] = "secret" },
                    Array.Empty<string>())));

        var json = JsonSerializer.Serialize(request, Options);
        using var document = JsonDocument.Parse(json);
        var transport = document.RootElement.GetProperty("transport");

        Assert.Equal("stdio", transport.GetProperty("kind").GetString());
        Assert.Equal("uvx", transport.GetProperty("command").GetString());
        Assert.Equal("project", transport.GetProperty("workingDirectory").GetString());
        Assert.Equal(15UL, transport.GetProperty("startupTimeoutSeconds").GetUInt64());
    }
}
