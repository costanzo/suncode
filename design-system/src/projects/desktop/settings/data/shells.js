// Review fixtures only. No executable is inspected or installed by this browser.
export const shellPlatforms = {
  windows: {
    agent: {
      path: "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
      version: "Windows PowerShell 5.1",
    },
    terminal: {
      path: "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
      version: "Windows PowerShell 5.1",
    },
    example: "C:\\Tools\\PowerShell\\pwsh.exe",
  },
  macos: {
    agent: { path: "/bin/sh", version: "POSIX sh" },
    terminal: { path: "/bin/zsh", version: "zsh 5.9" },
    example: "/bin/zsh",
  },
  linux: {
    agent: { path: "/bin/sh", version: "POSIX sh" },
    terminal: { path: "/bin/bash", version: "GNU bash 5.2" },
    example: "/bin/bash",
  },
};

export const shellReviewStates = [
  { value: "default", label: "System defaults" },
  { value: "installed", label: "PowerShell 7 installed" },
  { value: "missing", label: "Executable missing" },
  { value: "timeout", label: "Validation timeout" },
  { value: "unsupported", label: "Unsupported shell" },
  { value: "download-failed", label: "Download failed" },
  { value: "checksum-failed", label: "Verification failed" },
  { value: "install-failed", label: "Install directory denied" },
];

export function managedPowerShell(architecture) {
  return {
    path: `%LOCALAPPDATA%\\SunCode\\runtimes\\powershell\\7.6.0-${architecture}\\pwsh.exe`,
    version: "PowerShell 7.6.0",
  };
}

export const installFailures = {
  "download-failed": "Download failed. Check your connection and Network settings, then retry.",
  "checksum-failed":
    "The download could not be verified. It was discarded; retry with a new download.",
  "install-failed":
    "Cannot write to the installation directory. Check its permissions, then retry.",
};
