using System.Collections.Concurrent;
using System.Runtime.InteropServices;
using Tmds.DBus.Protocol;

namespace SunCode.Desktop.Infrastructure;

internal static class SystemNotificationBackend
{
    public static ISystemNotificationBackend Create() =>
        OperatingSystem.IsMacOS() ? new MacOSNotificationBackend()
        : OperatingSystem.IsWindows() ? new WindowsNotificationBackend()
        : OperatingSystem.IsLinux() ? new LinuxNotificationBackend()
        : new UnavailableNotificationBackend();
}

internal sealed class UnavailableNotificationBackend : ISystemNotificationBackend
{
    public event Action<DesktopActivationRequest>? Activated { add { } remove { } }
    public Task InitializeAsync(CancellationToken cancellationToken) => Task.CompletedTask;
    public Task ShowAsync(SystemNotification notification, CancellationToken cancellationToken) =>
        throw new PlatformNotSupportedException("System notifications are unavailable on this platform");
    public void Dispose() { }
}

internal sealed class WindowsNotificationBackend : ISystemNotificationBackend
{
    public event Action<DesktopActivationRequest>? Activated;

    public Task InitializeAsync(CancellationToken cancellationToken)
    {
        if (!OperatingSystem.IsWindows()) return Task.CompletedTask;
        WindowsToastBridge.Initialize(encoded =>
        {
            if (DesktopActivationRequest.TryParseLaunchArgument(encoded, out var request)) Activated?.Invoke(request!);
        });
        return Task.CompletedTask;
    }

    public Task ShowAsync(SystemNotification notification, CancellationToken cancellationToken)
    {
        WindowsToastBridge.Show(
            notification.Id,
            notification.Title,
            notification.Body,
            notification.Activation.ToLaunchArgument());
        return Task.CompletedTask;
    }

    public void Dispose()
    {
        WindowsToastBridge.Dispose();
        Activated = null;
    }
}

internal sealed class MacOSNotificationBackend : ISystemNotificationBackend
{
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void NativeActivationCallback(IntPtr activation);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void NativeDeliveryCallback(int delivered);

    private readonly NativeActivationCallback _callback;
    public event Action<DesktopActivationRequest>? Activated;

    public MacOSNotificationBackend() => _callback = OnActivated;

    public Task InitializeAsync(CancellationToken cancellationToken)
    {
        if (OperatingSystem.IsMacOS()) Native.Initialize(_callback);
        return Task.CompletedTask;
    }

    public async Task ShowAsync(SystemNotification notification, CancellationToken cancellationToken)
    {
        var completion = new TaskCompletionSource<bool>(TaskCreationOptions.RunContinuationsAsynchronously);
        NativeDeliveryCallback callback = delivered => completion.TrySetResult(delivered != 0);
        Native.Show(notification.Id, notification.Title, notification.Body, notification.Activation.ToLaunchArgument(), callback);
        var delivered = await completion.Task.WaitAsync(cancellationToken);
        GC.KeepAlive(callback);
        if (!delivered) throw new InvalidOperationException("macOS notification permission or delivery was unavailable");
    }

    private void OnActivated(IntPtr value)
    {
        var encoded = Marshal.PtrToStringUTF8(value);
        if (DesktopActivationRequest.TryParseLaunchArgument(encoded, out var request)) Activated?.Invoke(request!);
    }

    public void Dispose() => Activated = null;

    private static class Native
    {
        [DllImport("libsuncode_notifications.dylib", EntryPoint = "suncode_notifications_initialize", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void Initialize(NativeActivationCallback callback);

        [DllImport("libsuncode_notifications.dylib", EntryPoint = "suncode_notifications_show", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void Show(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string identifier,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string title,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string body,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string activation,
            NativeDeliveryCallback callback);
    }
}

internal sealed class LinuxNotificationBackend : ISystemNotificationBackend
{
    private const string Service = "org.freedesktop.Notifications";
    private const string Path = "/org/freedesktop/Notifications";
    private readonly ConcurrentDictionary<uint, DesktopActivationRequest> _activations = new();
    private DBusConnection? _connection;
    private IDisposable? _actionObserver;
    private bool _supportsActions;
    public event Action<DesktopActivationRequest>? Activated;

    public async Task InitializeAsync(CancellationToken cancellationToken)
    {
        _connection = new DBusConnection(DBusAddress.Session
            ?? throw new InvalidOperationException("D-Bus session address is unavailable"));
        await _connection.ConnectAsync();
        _supportsActions = (await CallStringArrayAsync("GetCapabilities")).Contains("actions", StringComparer.Ordinal);
        if (_supportsActions)
        {
            _actionObserver = await _connection.AddMatchAsync(
                new MatchRule { Type = MessageType.Signal, Interface = Service, Member = "ActionInvoked", Path = Path },
                static (message, _) =>
                {
                    var reader = message.GetBodyReader();
                    return (reader.ReadUInt32(), reader.ReadString());
                },
                notification =>
                {
                    if (!notification.HasValue) return;
                    var (id, action) = notification.Value;
                    if (action == "default" && _activations.TryRemove(id, out var request)) Activated?.Invoke(request);
                },
                emitOnCapturedContext: false);
        }
    }

    public async Task ShowAsync(SystemNotification notification, CancellationToken cancellationToken)
    {
        if (_connection is null) throw new InvalidOperationException("Linux notification service is not initialized");
        MessageBuffer message;
        {
            using var writer = _connection.GetMessageWriter();
            writer.WriteMethodCallHeader(Service, Path, Service, "Notify", "susssasa{sv}i", MessageFlags.None);
            writer.WriteString("SunCode");
            writer.WriteUInt32(0);
            writer.WriteString("suncode");
            writer.WriteString(notification.Title);
            writer.WriteString(notification.Body);
            writer.WriteArray(_supportsActions ? new[] { "default", "Open" } : Array.Empty<string>());
            writer.WriteDictionary(new Dictionary<string, VariantValue>
            {
                ["desktop-entry"] = "dev.suncode.desktop",
                ["x-canonical-private-synchronous"] = notification.Id
            });
            writer.WriteInt32(-1);
            message = writer.CreateMessage();
        }
        var id = await _connection.CallMethodAsync(message, static (reply, _) =>
        {
            var reader = reply.GetBodyReader();
            return reader.ReadUInt32();
        }, null);
        if (_supportsActions) _activations[id] = notification.Activation;
    }

    private async Task<string[]> CallStringArrayAsync(string member)
    {
        MessageBuffer message;
        {
            using var writer = _connection!.GetMessageWriter();
            writer.WriteMethodCallHeader(Service, Path, Service, member, string.Empty, MessageFlags.None);
            message = writer.CreateMessage();
        }
        return await _connection.CallMethodAsync(message, static (reply, _) =>
        {
            var reader = reply.GetBodyReader();
            return reader.ReadArrayOfString();
        }, null);
    }

    public void Dispose()
    {
        _actionObserver?.Dispose();
        _connection?.Dispose();
        _activations.Clear();
        Activated = null;
    }
}
