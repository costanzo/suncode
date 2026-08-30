using System.Runtime.InteropServices;

namespace SunCode.Desktop.Agent;

internal static class NativeMethods
{
    private const string Library = "suncode_agent";

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    internal delegate void EventCallback(IntPtr eventJson, IntPtr userData);

    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern uint suncode_agent_sdk_abi_version();
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_open_default(out IntPtr errorOut);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern void suncode_agent_sdk_close(IntPtr handle);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_health(IntPtr handle);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_diagnostics(IntPtr handle);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_list_models(IntPtr handle);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_list_settings(IntPtr handle, IntPtr projectId, IntPtr sessionId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_set_setting(IntPtr handle, IntPtr scope, IntPtr projectId, IntPtr sessionId, IntPtr key, IntPtr valueJson);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_list_credentials(IntPtr handle);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_set_credential(IntPtr handle, IntPtr provider, IntPtr apiKey);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_remove_credential(IntPtr handle, IntPtr provider);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_set_provider_endpoint(IntPtr handle, IntPtr provider, IntPtr endpoint);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_list_projects(IntPtr handle);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_open_project(IntPtr handle, IntPtr path, IntPtr displayName);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_select_project(IntPtr handle, IntPtr projectId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_list_project_dependencies(IntPtr handle, IntPtr projectId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_add_project_dependency(IntPtr handle, IntPtr projectId, IntPtr path);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_remove_project_dependency(IntPtr handle, IntPtr projectId, IntPtr dependencyId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_list_project_directory(IntPtr handle, IntPtr projectId, IntPtr dependencyId, IntPtr path);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_git_status(IntPtr handle, IntPtr projectId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_git_diff_file(IntPtr handle, IntPtr projectId, IntPtr scope, IntPtr path);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_list_sessions(IntPtr handle, IntPtr projectId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_create_session(IntPtr handle, IntPtr projectId, IntPtr title, IntPtr model);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_rename_session(IntPtr handle, IntPtr sessionId, IntPtr title);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_archive_session(IntPtr handle, IntPtr sessionId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_set_session_pinned(IntPtr handle, IntPtr sessionId, byte pinned);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_list_session_images(IntPtr handle, IntPtr sessionId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_add_session_image(IntPtr handle, IntPtr sessionId, IntPtr imageJson);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_remove_session_image(IntPtr handle, IntPtr sessionId, IntPtr imageId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_session_snapshot(IntPtr handle, IntPtr sessionId, long after);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_session_usage(IntPtr handle, IntPtr sessionId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_list_provider_exchanges(IntPtr handle, IntPtr sessionId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_provider_exchange(IntPtr handle, IntPtr sessionId, IntPtr exchangeId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_list_checkpoints(IntPtr handle, IntPtr sessionId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_restore_checkpoint(IntPtr handle, IntPtr manifestId, IntPtr sessionId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_submit_turn(IntPtr handle, IntPtr sessionId, IntPtr input, IntPtr idempotencyKey, IntPtr model, IntPtr reasoningEffort);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_submit_turn_with_attachments(IntPtr handle, IntPtr sessionId, IntPtr input, IntPtr idempotencyKey, IntPtr model, IntPtr reasoningEffort, IntPtr imageIdsJson);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_cancel_turn(IntPtr handle, IntPtr sessionId, IntPtr turnId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_resolve_approval(IntPtr handle, IntPtr approvalId, IntPtr decision);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_reply_question(IntPtr handle, IntPtr requestId, IntPtr answersJson);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_reject_question(IntPtr handle, IntPtr requestId);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern IntPtr suncode_agent_sdk_subscribe_session(IntPtr handle, IntPtr sessionId, long after, EventCallback callback, IntPtr userData, out IntPtr errorOut);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern void suncode_agent_sdk_subscription_close(IntPtr subscription);
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] internal static extern void suncode_agent_sdk_string_free(IntPtr value);
}
