using System.Buffers.Binary;
using System.IO.Pipes;
using System.Net.Sockets;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Runtime.Versioning;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace SunCode.Desktop.Infrastructure;

internal sealed record DesktopActivationRequest(
    int Version,
    string RequestId,
    string Kind,
    string? ProjectId,
    string? SessionId,
    string? ParentSessionId,
    string? ChildSessionId,
    string? CorrelationId,
    string Source)
{
    public const int CurrentVersion = 1;

    public static DesktopActivationRequest Application(string source = "launch") => new(
        CurrentVersion, Guid.NewGuid().ToString("N"), "activate.application",
        null, null, null, null, null, source);

    public static DesktopActivationRequest Session(
        string projectId,
        string sessionId,
        string correlationId,
        string source,
        string? parentSessionId = null,
        string? childSessionId = null) => new(
            CurrentVersion, Guid.NewGuid().ToString("N"), "activate.session",
            projectId, sessionId, parentSessionId, childSessionId, correlationId, source);

    public void Validate()
    {
        if (Version != CurrentVersion) throw new InvalidDataException("Unsupported activation protocol version");
        ValidateToken(RequestId, nameof(RequestId), required: true);
        if (Kind is not ("activate.application" or "activate.session"))
            throw new InvalidDataException("Unsupported activation kind");
        ValidateToken(Source, nameof(Source), required: true);
        ValidateToken(ProjectId, nameof(ProjectId), Kind == "activate.session");
        ValidateToken(SessionId, nameof(SessionId), Kind == "activate.session");
        ValidateToken(ParentSessionId, nameof(ParentSessionId), false);
        ValidateToken(ChildSessionId, nameof(ChildSessionId), false);
        ValidateToken(CorrelationId, nameof(CorrelationId), Kind == "activate.session");
        if ((ParentSessionId is null) != (ChildSessionId is null))
            throw new InvalidDataException("Parent and child session IDs must be supplied together");
        if (ChildSessionId is not null && SessionId != ChildSessionId)
            throw new InvalidDataException("Child activation session ID does not match child session ID");
        if (ChildSessionId is not null && ParentSessionId == ChildSessionId)
            throw new InvalidDataException("Parent and child session IDs must differ");
    }

    public string ToLaunchArgument()
    {
        var json = JsonSerializer.Serialize(this, JsonOptions);
        return Convert.ToBase64String(Encoding.UTF8.GetBytes(json))
            .TrimEnd('=').Replace('+', '-').Replace('/', '_');
    }

    public static bool TryParseLaunchArgument(string? value, out DesktopActivationRequest? request)
    {
        request = null;
        if (string.IsNullOrWhiteSpace(value) || value.Length > 8192) return false;
        try
        {
            var encoded = value.Replace('-', '+').Replace('_', '/');
            encoded = encoded.PadRight(encoded.Length + ((4 - encoded.Length % 4) % 4), '=');
            request = JsonSerializer.Deserialize<DesktopActivationRequest>(
                Convert.FromBase64String(encoded), JsonOptions);
            request?.Validate();
            return request is not null;
        }
        catch
        {
            request = null;
            return false;
        }
    }

    private static void ValidateToken(string? value, string name, bool required)
    {
        if (required && string.IsNullOrWhiteSpace(value)) throw new InvalidDataException($"{name} is required");
        if (value is { Length: > 256 }) throw new InvalidDataException($"{name} is too long");
        if (value?.Any(char.IsControl) == true) throw new InvalidDataException($"{name} contains control characters");
    }

    internal static readonly JsonSerializerOptions JsonOptions = new(JsonSerializerDefaults.Web);
}

internal sealed record DesktopActivationAck(int Version, string RequestId, bool Accepted, string? Error);

internal sealed class DesktopInstanceCoordinator : IDisposable
{
    internal const int MaximumFrameBytes = 16 * 1024;
    private readonly string _endpointName;
    private readonly string? _socketPath;
    private readonly string? _lockPath;
    private readonly Mutex? _mutex;
    private readonly FileStream? _lockFile;
    private readonly CancellationTokenSource _shutdown = new();
    private readonly Queue<DesktopActivationRequest> _pending = new();
    private readonly SemaphoreSlim _dispatchGate = new(1, 1);
    private readonly object _pendingGate = new();
    private Task? _server;
    private bool _disposed;

    private DesktopInstanceCoordinator(
        bool isPrimary,
        string endpointName,
        string? socketPath,
        string? lockPath,
        Mutex? mutex,
        FileStream? lockFile)
    {
        IsPrimary = isPrimary;
        _endpointName = endpointName;
        _socketPath = socketPath;
        _lockPath = lockPath;
        _mutex = mutex;
        _lockFile = lockFile;
    }

    public bool IsPrimary { get; }
    public event Action<DesktopActivationRequest>? ActivationReceived;

    public static DesktopInstanceCoordinator Acquire(string dataDirectory)
    {
        var fullPath = Path.GetFullPath(dataDirectory).TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar);
        var identity = OperatingSystem.IsWindows() ? fullPath.ToUpperInvariant() : fullPath;
        var hash = Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(identity))).ToLowerInvariant()[..24];
        var endpoint = $"suncode-desktop-{hash}";
        if (OperatingSystem.IsWindows())
        {
            var mutex = new Mutex(true, $"Local\\{endpoint}", out var created);
            return new DesktopInstanceCoordinator(created, endpoint, null, null, mutex, null);
        }

        Directory.CreateDirectory(fullPath);
        var lockPath = Path.Combine(fullPath, "desktop-instance.lock");
        FileStream? lockFile = null;
        var primary = false;
        try
        {
            lockFile = new FileStream(lockPath, FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.ReadWrite);
            if (!UnixFileLock.TryAcquire(lockFile.SafeFileHandle)) throw new IOException("Desktop instance lock is held");
            primary = true;
        }
        catch (IOException)
        {
            lockFile?.Dispose();
            lockFile = null;
        }
        if (primary) File.SetUnixFileMode(lockPath, UnixFileMode.UserRead | UnixFileMode.UserWrite);
        var runtime = Environment.GetEnvironmentVariable("XDG_RUNTIME_DIR");
        if (string.IsNullOrWhiteSpace(runtime)) runtime = Path.GetTempPath();
        var socketPath = Path.Combine(runtime, $"{endpoint}.sock");
        ValidatePrivateExistingFile(socketPath);
        return new DesktopInstanceCoordinator(primary, endpoint, socketPath, lockPath, null, lockFile);
    }

    [UnsupportedOSPlatform("windows")]
    private static void ValidatePrivateExistingFile(string path)
    {
        if (!File.Exists(path)) return;
        var mode = File.GetUnixFileMode(path);
        const UnixFileMode unsafeMode = UnixFileMode.GroupRead | UnixFileMode.GroupWrite | UnixFileMode.GroupExecute
            | UnixFileMode.OtherRead | UnixFileMode.OtherWrite | UnixFileMode.OtherExecute;
        if ((mode & unsafeMode) != 0) throw new UnauthorizedAccessException("Desktop activation endpoint permissions are unsafe");
    }

    public void StartServer()
    {
        if (!IsPrimary || _server is not null) return;
        if (_socketPath is not null && File.Exists(_socketPath)) File.Delete(_socketPath);
        _server = OperatingSystem.IsWindows()
            ? RunNamedPipeServerAsync(_shutdown.Token)
            : RunUnixSocketServerAsync(_shutdown.Token);
    }

    public IReadOnlyList<DesktopActivationRequest> DrainPending()
    {
        lock (_pendingGate)
        {
            var items = _pending.ToArray();
            _pending.Clear();
            return items;
        }
    }

    public async Task<bool> ForwardAsync(DesktopActivationRequest request, CancellationToken cancellationToken = default)
    {
        request.Validate();
        using var timeout = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        timeout.CancelAfter(TimeSpan.FromSeconds(4));
        for (var attempt = 0; attempt < 20; attempt++)
        {
            try
            {
                await using var stream = await ConnectAsync(timeout.Token).ConfigureAwait(false);
                await WriteFrameAsync(stream, request, timeout.Token).ConfigureAwait(false);
                var ack = await ReadFrameAsync<DesktopActivationAck>(stream, timeout.Token).ConfigureAwait(false);
                return ack.Version == DesktopActivationRequest.CurrentVersion
                    && ack.RequestId == request.RequestId && ack.Accepted;
            }
            catch (Exception exception) when (exception is IOException or SocketException or TimeoutException)
            {
                if (attempt == 19) return false;
                await Task.Delay(100, timeout.Token).ConfigureAwait(false);
            }
        }
        return false;
    }

    private async ValueTask<Stream> ConnectAsync(CancellationToken cancellationToken)
    {
        if (OperatingSystem.IsWindows())
        {
            var pipe = new NamedPipeClientStream(".", _endpointName, PipeDirection.InOut, PipeOptions.Asynchronous);
            await pipe.ConnectAsync(cancellationToken).ConfigureAwait(false);
            return pipe;
        }
        var socket = new Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified);
        try
        {
            await socket.ConnectAsync(new UnixDomainSocketEndPoint(_socketPath!), cancellationToken).ConfigureAwait(false);
            return new NetworkStream(socket, ownsSocket: true);
        }
        catch
        {
            socket.Dispose();
            throw;
        }
    }

    private async Task RunNamedPipeServerAsync(CancellationToken cancellationToken)
    {
        while (!cancellationToken.IsCancellationRequested)
        {
            await using var pipe = new NamedPipeServerStream(
                _endpointName, PipeDirection.InOut, 1, PipeTransmissionMode.Byte,
                PipeOptions.Asynchronous | PipeOptions.CurrentUserOnly);
            try
            {
                await pipe.WaitForConnectionAsync(cancellationToken).ConfigureAwait(false);
                await HandleClientAsync(pipe, cancellationToken).ConfigureAwait(false);
            }
            catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested) { }
            catch (Exception exception) { DiagnosticLog.Warn("desktop.ipc", $"transport=pipe error={exception.Message}"); }
        }
    }

    [UnsupportedOSPlatform("windows")]
    private async Task RunUnixSocketServerAsync(CancellationToken cancellationToken)
    {
        using var listener = new Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified);
        listener.Bind(new UnixDomainSocketEndPoint(_socketPath!));
        File.SetUnixFileMode(_socketPath!, UnixFileMode.UserRead | UnixFileMode.UserWrite);
        listener.Listen(8);
        try
        {
            while (!cancellationToken.IsCancellationRequested)
            {
                try
                {
                    var socket = await listener.AcceptAsync(cancellationToken).ConfigureAwait(false);
                    _ = Task.Run(async () =>
                    {
                        await using var stream = new NetworkStream(socket, ownsSocket: true);
                        await HandleClientAsync(stream, cancellationToken).ConfigureAwait(false);
                    }, cancellationToken);
                }
                catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested) { }
                catch (Exception exception) { DiagnosticLog.Warn("desktop.ipc", $"transport=unix error={exception.Message}"); }
            }
        }
        finally
        {
            if (_socketPath is not null && File.Exists(_socketPath)) File.Delete(_socketPath);
        }
    }

    private async Task HandleClientAsync(Stream stream, CancellationToken cancellationToken)
    {
        DesktopActivationAck ack;
        try
        {
            var request = await ReadFrameAsync<DesktopActivationRequest>(stream, cancellationToken).ConfigureAwait(false);
            request.Validate();
            await RejectTrailingFrameAsync(stream, cancellationToken).ConfigureAwait(false);
            await _dispatchGate.WaitAsync(cancellationToken).ConfigureAwait(false);
            try { Dispatch(request); }
            finally { _dispatchGate.Release(); }
            ack = new DesktopActivationAck(DesktopActivationRequest.CurrentVersion, request.RequestId, true, null);
        }
        catch (Exception exception)
        {
            ack = new DesktopActivationAck(DesktopActivationRequest.CurrentVersion, string.Empty, false, exception.Message);
        }
        await WriteFrameAsync(stream, ack, cancellationToken).ConfigureAwait(false);
    }

    private static async Task RejectTrailingFrameAsync(Stream stream, CancellationToken cancellationToken)
    {
        using var timeout = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        timeout.CancelAfter(TimeSpan.FromMilliseconds(25));
        var probe = new byte[1];
        try
        {
            var read = await stream.ReadAsync(probe, timeout.Token).ConfigureAwait(false);
            if (read != 0) throw new InvalidDataException("Activation connection contains trailing data");
        }
        catch (OperationCanceledException) when (!cancellationToken.IsCancellationRequested)
        {
            // The valid one-request connection remains open while the client waits for its ACK.
        }
    }

    private void Dispatch(DesktopActivationRequest request)
    {
        var handler = ActivationReceived;
        if (handler is not null)
        {
            handler(request);
            return;
        }
        lock (_pendingGate)
        {
            if (_pending.Count >= 32) _pending.Dequeue();
            _pending.Enqueue(request);
        }
    }

    internal static async Task WriteFrameAsync<T>(Stream stream, T value, CancellationToken cancellationToken)
    {
        var payload = JsonSerializer.SerializeToUtf8Bytes(value, DesktopActivationRequest.JsonOptions);
        if (payload.Length is 0 or > MaximumFrameBytes) throw new InvalidDataException("Activation frame is invalid");
        var header = new byte[4];
        BinaryPrimitives.WriteInt32BigEndian(header, payload.Length);
        await stream.WriteAsync(header, cancellationToken).ConfigureAwait(false);
        await stream.WriteAsync(payload, cancellationToken).ConfigureAwait(false);
        await stream.FlushAsync(cancellationToken).ConfigureAwait(false);
    }

    internal static async Task<T> ReadFrameAsync<T>(Stream stream, CancellationToken cancellationToken)
    {
        var header = new byte[4];
        await stream.ReadExactlyAsync(header, cancellationToken).ConfigureAwait(false);
        var length = BinaryPrimitives.ReadInt32BigEndian(header);
        if (length is <= 0 or > MaximumFrameBytes) throw new InvalidDataException("Activation frame is too large");
        var payload = new byte[length];
        await stream.ReadExactlyAsync(payload, cancellationToken).ConfigureAwait(false);
        return JsonSerializer.Deserialize<T>(payload, DesktopActivationRequest.JsonOptions)
            ?? throw new InvalidDataException("Activation frame is empty");
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        _shutdown.Cancel();
        _lockFile?.Dispose();
        if (IsPrimary && _lockPath is not null)
        {
            try { File.Delete(_lockPath); } catch (IOException) { }
        }
        if (IsPrimary) _mutex?.ReleaseMutex();
        _mutex?.Dispose();
        _shutdown.Dispose();
    }

    [UnsupportedOSPlatform("windows")]
    private static class UnixFileLock
    {
        private const int Exclusive = 2;
        private const int NonBlocking = 4;

        [DllImport("libc", SetLastError = true)]
        private static extern int flock(int fileDescriptor, int operation);

        public static bool TryAcquire(SafeFileHandle handle) =>
            flock(handle.DangerousGetHandle().ToInt32(), Exclusive | NonBlocking) == 0;
    }
}
