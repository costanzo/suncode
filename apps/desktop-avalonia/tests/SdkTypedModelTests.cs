using System.Text.Json;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class SdkTypedModelTests
{
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
    }
}
