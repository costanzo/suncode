#if WINDOWS
using Microsoft.Toolkit.Uwp.Notifications;
using Microsoft.Win32;
#endif

namespace SunCode.Desktop.Infrastructure;

internal static class WindowsToastBridge
{
#if WINDOWS
    private static Action<string>? _activated;

    public static void Initialize(Action<string> activated)
    {
        _activated = activated;
        RegisterProtocol();
        ToastNotificationManagerCompat.OnActivated += OnActivated;
    }

    public static void Show(string id, string title, string body, string activation)
    {
        var uri = new Uri($"suncode://activate/session?payload={Uri.EscapeDataString(activation)}");
        new ToastContentBuilder()
            .AddText(title)
            .AddText(body)
            .SetProtocolActivation(uri)
            .Show(toast =>
            {
                toast.Tag = StableTag(id);
                toast.Group = "suncode-attention";
                toast.ExpirationTime = DateTimeOffset.Now.AddDays(7);
            });
    }

    private static void OnActivated(ToastNotificationActivatedEventArgsCompat args)
    {
        var values = ToastArguments.Parse(args.Argument);
        if (values.TryGetValue("payload", out var payload)) _activated?.Invoke(payload);
    }

    private static void RegisterProtocol()
    {
        var executable = Environment.ProcessPath
            ?? throw new InvalidOperationException("The SunCode executable path is unavailable");
        using var scheme = Registry.CurrentUser.CreateSubKey(@"Software\Classes\suncode");
        scheme.SetValue(null, "URL:SunCode Activation Protocol");
        scheme.SetValue("URL Protocol", string.Empty);
        using var command = scheme.CreateSubKey(@"shell\open\command");
        command.SetValue(null, $"\"{executable}\" \"%1\"");
    }

    private static string StableTag(string value) =>
        Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(System.Text.Encoding.UTF8.GetBytes(value)))
            .ToLowerInvariant()[..16];

    public static void Dispose()
    {
        ToastNotificationManagerCompat.OnActivated -= OnActivated;
        _activated = null;
    }
#else
    public static void Initialize(Action<string> activated) =>
        throw new PlatformNotSupportedException();

    public static void Show(string id, string title, string body, string activation) =>
        throw new PlatformNotSupportedException();

    public static void Dispose() { }
#endif
}
