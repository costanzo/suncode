namespace SunCode.Desktop.Agent;

public sealed class SdkException(string code, string message) : Exception(message)
{
    public string Code { get; } = code;
}
