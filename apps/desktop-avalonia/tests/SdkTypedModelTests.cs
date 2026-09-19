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

        const string sessionJson = "{\"sessionId\":\"session-1\",\"projectId\":\"project-1\",\"title\":\"Chat\",\"modelId\":\"gpt-5.5\",\"reasoningEffort\":\"high\",\"kind\":\"primary\",\"parentSessionId\":null,\"agentId\":null,\"agentVersion\":null,\"status\":\"active\",\"createdAt\":\"now\",\"updatedAt\":\"now\",\"lastActivityAt\":\"now\",\"archivedAt\":null,\"pinAt\":null}";
        var session = JsonSerializer.Deserialize<SessionRecord>(sessionJson, Options);

        Assert.NotNull(session);
        Assert.Equal("gpt-5.5", session.ModelId);
        Assert.Equal("high", session.ReasoningEffort);
        Assert.Equal("primary", session.Kind);
    }

    [Fact]
    public void Deserializes_built_in_agent_and_child_session_contracts()
    {
        const string agentsJson = """
            {"agents":[{"id":"builtin.swe.v1","name":"swe-agent","displayName":"Software Engineering Agent","description":"Implements software changes.","version":1,"allowedTools":["read","write"],"modelPolicy":"Inherits parent session model","mcpPolicy":"Not allowed","canDelegate":false,"toolCallLimit":64}]}
            """;
        var agents = JsonSerializer.Deserialize<AgentsResult>(agentsJson, Options);

        var agent = Assert.Single(Assert.IsType<AgentsResult>(agents).Agents);
        Assert.Equal("swe-agent", agent.Name);
        Assert.False(agent.CanDelegate);
        Assert.Equal((uint)64, agent.ToolCallLimit);

        const string childrenJson = """
            {"parentSessionId":"parent-1","sessions":[{"sessionId":"child-1","projectId":"project-1","title":"Implement","modelId":"gpt-5.5","reasoningEffort":"high","kind":"child","parentSessionId":"parent-1","agentId":"builtin.swe.v1","agentVersion":1,"status":"active","createdAt":"created","updatedAt":"updated","lastActivityAt":"updated","archivedAt":null,"pinAt":null}],"sessionStates":{"child-1":"approval"},"invocations":[{"invocationId":"invocation-1","parentSessionId":"parent-1","parentTurnId":"turn-1","parentToolCallId":"call-1","childSessionId":"child-1","agentId":"builtin.swe.v1","agentVersion":1,"task":{"text":"Implement"},"allowedTools":["read","write"],"modelId":"gpt-5.5","state":"awaiting_approval","result":{"approvalId":"approval-1"},"errorCode":null,"createdAt":"created","startedAt":"started","completedAt":null}]}
            """;
        var children = JsonSerializer.Deserialize<ChildSessionsResult>(childrenJson, Options);

        Assert.NotNull(children);
        Assert.Equal("child", Assert.Single(children.Sessions).Kind);
        Assert.Equal("awaiting_approval", Assert.Single(children.Invocations).State);
        Assert.Equal("approval", children.SessionStates["child-1"]);
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

    [Fact]
    public void Serializes_language_server_request_with_write_only_environment_patch()
    {
        using var optionsDocument = JsonDocument.Parse("{\"check\":{\"command\":\"clippy\"}}");
        var request = new LanguageServerWriteRequest(
            "Rust Analyzer",
            "rust-analyzer",
            Array.Empty<string>(),
            ["rust"],
            ["Cargo.toml"],
            optionsDocument.RootElement.Clone(),
            new LanguageServerEnvironmentChanges(
                new Dictionary<string, string> { ["RUST_LOG"] = "warn" },
                Array.Empty<string>()));

        var json = JsonSerializer.Serialize(request, Options);
        using var document = JsonDocument.Parse(json);

        Assert.Equal("Rust Analyzer", document.RootElement.GetProperty("displayName").GetString());
        Assert.Equal("rust-analyzer", document.RootElement.GetProperty("command").GetString());
        Assert.Equal("rust", document.RootElement.GetProperty("languageIds")[0].GetString());
        Assert.Equal("warn", document.RootElement.GetProperty("environment").GetProperty("set").GetProperty("RUST_LOG").GetString());
        Assert.Equal(30UL, document.RootElement.GetProperty("startupTimeoutSeconds").GetUInt64());
    }

    [Fact]
    public void Serializes_proxy_configuration_with_write_only_password_patch()
    {
        var request = new ProxyConfigurationRequest(
            "custom",
            "http://proxy.example.test:8080",
            "developer",
            "secret",
            false,
            [".internal.example.test", "10.0.0.0/8"]);

        var json = JsonSerializer.Serialize(request, Options);
        using var document = JsonDocument.Parse(json);

        Assert.Equal("custom", document.RootElement.GetProperty("mode").GetString());
        Assert.Equal("secret", document.RootElement.GetProperty("password").GetString());
        Assert.False(document.RootElement.GetProperty("clearPassword").GetBoolean());

        const string responseJson = """
            {"mode":"custom","url":"http://proxy.example.test:8080","username":"developer","passwordConfigured":true,"bypass":[".internal.example.test"]}
            """;
        var result = JsonSerializer.Deserialize<ProxyConfigurationResult>(responseJson, Options);

        Assert.NotNull(result);
        Assert.True(result.PasswordConfigured);
        Assert.Equal("developer", result.Username);
    }

    [Fact]
    public void Deserializes_rust_camel_case_dependency_checkpoint_and_approval_fields()
    {
        const string dependenciesJson = """
            {"projectId":"project-1","dependencies":[{"dependencyId":"dependency-1","projectId":"project-1","displayName":"Library","createdAt":"created"}]}
            """;
        var dependencies = JsonSerializer.Deserialize<ProjectDependenciesResult>(dependenciesJson, Options);

        Assert.NotNull(dependencies);
        Assert.Equal("project-1", dependencies.ProjectId);
        Assert.Equal("dependency-1", Assert.Single(dependencies.Dependencies).DependencyId);

        const string checkpointJson = """
            {"manifest":{"manifestId":"manifest-1","sessionId":"session-1","turnId":"turn-1","status":"available","createdAt":"created","updatedAt":"updated","expiresAt":"expires","restoredAt":null},"items":[{"checkpointId":"checkpoint-1","manifestId":"manifest-1","sessionId":"session-1","turnId":"turn-1","toolCallId":"tool-1","relativePath":"src/main.rs","status":"available","createdAt":"created","restoredAt":null,"invalidatedAt":null,"ordinal":0}]}
            """;
        var checkpoint = JsonSerializer.Deserialize<CheckpointDetails>(checkpointJson, Options);

        Assert.NotNull(checkpoint);
        Assert.Equal("manifest-1", checkpoint.Manifest.ManifestId);
        Assert.Equal("src/main.rs", Assert.Single(checkpoint.Items).RelativePath);

        const string approvalJson = """
            {"approvalId":"approval-1","projectId":"project-1","sessionId":"session-1","turnId":"turn-1","toolCallId":"tool-1","operation":"write","arguments":{},"status":"pending","decision":null,"decisionSource":null,"createdAt":"created","updatedAt":"updated"}
            """;
        var approval = JsonSerializer.Deserialize<ApprovalRecord>(approvalJson, Options);

        Assert.NotNull(approval);
        Assert.Equal("approval-1", approval.ApprovalId);
        Assert.Equal("tool-1", approval.ToolCallId);
    }

    [Fact]
    public void Deserializes_rust_provider_result_and_event_payload_fields()
    {
        const string exchangesJson = """
            {"session_id":"session-1","turns":[],"exchanges":[]}
            """;
        var exchanges = JsonSerializer.Deserialize<ProviderExchangesResult>(exchangesJson, Options);

        Assert.NotNull(exchanges);
        Assert.Equal("session-1", exchanges.SessionId);

        const string eventJson = """
            {"session_id":"session-1","occurred_at":"now","event_type":"context.compacted","payload":{"turn_id":"turn-1","iteration":2,"tool_calls":3,"original_characters":100,"retained_characters":40,"original_tokens":25,"retained_tokens":10,"dropped_messages":4}}
            """;
        var agentEvent = JsonSerializer.Deserialize<AgentEvent>(eventJson, Options);

        Assert.NotNull(agentEvent);
        Assert.Equal((uint)2, agentEvent.Payload.Iteration);
        Assert.Equal(3, agentEvent.Payload.ToolCalls?.GetInt32());
        Assert.Equal(100, agentEvent.Payload.OriginalCharacters);
        Assert.Equal(4, agentEvent.Payload.DroppedMessages);
    }
}
