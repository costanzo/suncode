namespace SunCode.Sdk;

public sealed class SdkException(string code, string message) : Exception(message)
{
    public string Code { get; } = code;
}
