using System.Runtime.InteropServices;

namespace SunCode.Desktop.Runtime;

internal static class NativeMethods
{
    private const string Library = "suncode_runtime";

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    internal delegate void EventCallback(IntPtr eventJson, IntPtr userData);

    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern uint suncode_runtime_sdk_abi_version();
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_open_default(out IntPtr errorOut);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern void suncode_runtime_sdk_close(IntPtr handle);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_health(IntPtr handle);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_diagnostics(IntPtr handle);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_list_models(IntPtr handle);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_list_settings(IntPtr handle, IntPtr projectId, IntPtr sessionId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_set_setting(IntPtr handle, IntPtr scope, IntPtr projectId, IntPtr sessionId, IntPtr key, IntPtr valueJson);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_list_credentials(IntPtr handle);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_set_credential(IntPtr handle, IntPtr provider, IntPtr apiKey);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_remove_credential(IntPtr handle, IntPtr provider);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_list_projects(IntPtr handle);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_open_project(IntPtr handle, IntPtr path, IntPtr displayName);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_select_project(IntPtr handle, IntPtr projectId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_git_status(IntPtr handle, IntPtr projectId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_git_diff_file(IntPtr handle, IntPtr projectId, IntPtr scope, IntPtr path);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_list_sessions(IntPtr handle, IntPtr projectId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_create_session(IntPtr handle, IntPtr projectId, IntPtr title, IntPtr model);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_rename_session(IntPtr handle, IntPtr sessionId, IntPtr title);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_archive_session(IntPtr handle, IntPtr sessionId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_session_snapshot(IntPtr handle, IntPtr sessionId, long after);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_session_usage(IntPtr handle, IntPtr sessionId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_list_provider_exchanges(IntPtr handle, IntPtr sessionId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_provider_exchange(IntPtr handle, IntPtr sessionId, IntPtr exchangeId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_list_checkpoints(IntPtr handle, IntPtr sessionId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_restore_checkpoint(IntPtr handle, IntPtr manifestId, IntPtr sessionId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_submit_turn(IntPtr handle, IntPtr sessionId, IntPtr input, IntPtr idempotencyKey, IntPtr model);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_cancel_turn(IntPtr handle, IntPtr sessionId, IntPtr turnId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_resolve_approval(IntPtr handle, IntPtr approvalId, IntPtr decision);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_runtime_sdk_subscribe_session(IntPtr handle, IntPtr sessionId, long after, EventCallback callback, IntPtr userData, out IntPtr errorOut);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern void suncode_runtime_sdk_subscription_close(IntPtr subscription);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern void suncode_runtime_sdk_string_free(IntPtr value);
}
