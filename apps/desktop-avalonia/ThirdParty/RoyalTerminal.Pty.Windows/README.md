# RoyalTerminal Windows PTY source backend

This client-local project provides the `RoyalApps.RoyalTerminal.Terminal.Pty.Windows` 0.7.2 backend. Its package ID and assembly identity match the upstream package so RoyalTerminal's platform factory selects it.

`WindowsPty.cs`, `WindowsPtyEnvironment.cs`, `SafePseudoConsoleHandle.cs`, and `LICENSE` follow [RoyalTerminal commit c5e5365c08733ea7df42eec98fd505f1cfcef70b](https://github.com/royalapplications/RoyalTerminal/tree/c5e5365c08733ea7df42eec98fd505f1cfcef70b), the source revision recorded by the 0.7.2 package. The MIT license and upstream source headers are retained. These source files match upstream.

The 0.7.2 release incorporates the Windows 10 correction previously carried locally. It sets `STARTF_USESTDHANDLES` with null standard handles before `CreateProcessW`, allowing ConPTY to supply the console handles. It also leases and asynchronously releases the pseudoconsole through `SafePseudoConsoleHandle` to preserve output draining during process exit and resize races.

The build copies the license to `ThirdPartyNotices/RoyalTerminal.Pty.Windows.LICENSE` in application output and publish directories.

The source-backed integration is retained until the Windows terminal integration tests run against the official 0.7.2 package on Windows. After that verification, the project reference can be replaced with the official package. Do not edit the NuGet cache or patch a deployed binary.
