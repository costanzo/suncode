# RoyalTerminal Windows PTY compatibility patch

This client-local project replaces only the `RoyalApps.RoyalTerminal.Terminal.Pty.Windows` 0.7.0 transitive package. Its package ID and assembly identity match that package so RoyalTerminal's existing platform factory uses it without changing the Avalonia control or Unix backend.

`WindowsPty.cs`, `WindowsPtyEnvironment.cs`, and `LICENSE` are copied from [RoyalTerminal commit 4f5d6689ddb792a6e2c3ded923e198cad3d62012](https://github.com/royalapplications/RoyalTerminal/tree/4f5d6689ddb792a6e2c3ded923e198cad3d62012), the source revision recorded by the 0.7.0 package. The MIT license and upstream headers are retained.

The build copies the license to `ThirdPartyNotices/RoyalTerminal.Pty.Windows.LICENSE` in application output and publish directories.

The sole source patch sets `STARTF_USESTDHANDLES` with null standard handles before `CreateProcessW`. On Windows 10, leaving the flag unset can preserve the parent's standard handles even when the child is attached to a pseudoconsole. The shell then writes outside the terminal and cannot read terminal input. Explicit null handles let ConPTY supply the console handles. Do not set these handles to the host-side PTY pipes, enable handle inheritance, or change the host process's global standard handles.

Remove this override when an official release includes the correction, after running the Windows terminal integration tests against that release. Do not edit the NuGet cache or patch a deployed binary. Upstream code outside this compatibility correction remains unchanged.
