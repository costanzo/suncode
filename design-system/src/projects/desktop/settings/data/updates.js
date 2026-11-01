// Illustrative releases and states for design review. No update service is contacted.
export const updateComponents = [
  { id: "desktop", label: "Desktop", installed: "0.1.0", target: "0.1.1", size: 86 },
  { id: "sdk", label: "Agent SDK", installed: "0.1.0", target: "0.1.1", size: 18 },
];

export const updateScenarios = [
  { value: "idle", label: "Not checked", phase: "idle" },
  { value: "checking", label: "Checking", phase: "checking" },
  { value: "current", label: "Up to date", phase: "current" },
  { value: "desktop", label: "Desktop only", phase: "available", kind: "desktop" },
  { value: "sdk", label: "Agent SDK only", phase: "available" },
  { value: "paired", label: "Both components required", phase: "available", kind: "paired" },
  { value: "downloading", label: "Downloading", phase: "downloading" },
  { value: "verifying", label: "Verifying", phase: "verifying" },
  { value: "ready", label: "Ready to restart", phase: "ready" },
  { value: "busy", label: "Restart blocked by active work", phase: "busy" },
  { value: "check-failed", label: "Check failed", phase: "check-failed" },
  { value: "download-failed", label: "Download failed", phase: "download-failed" },
  { value: "verification-failed", label: "Verification failed", phase: "verification-failed" },
  { value: "install-failed", label: "Installation failed", phase: "install-failed" },
  { value: "restored", label: "Previous version restored", phase: "restored" },
  { value: "missing", label: "Agent SDK unavailable", phase: "missing" },
  { value: "unconfigured", label: "Update delivery not configured", phase: "unconfigured" },
  { value: "unmanaged", label: "Automatic installation unavailable", phase: "unmanaged" },
  { value: "check-timeout", label: "Check timed out", phase: "check-timeout" },
  { value: "download-timeout", label: "Download timed out", phase: "download-timeout" },
  { value: "download-stalled", label: "Download stopped responding", phase: "download-stalled" },
];

export const updateMessages = {
  "check-timeout": [
    "Checking for updates timed out",
    "The update service took too long to respond. Check your connection and try again.",
  ],
  "download-timeout": [
    "The download timed out",
    "The download took too long. Your installed versions have not changed. Try downloading again.",
  ],
  "download-stalled": [
    "The download stopped responding",
    "No data was received for 30 seconds. Your installed versions have not changed. Try downloading again.",
  ],
  unmanaged: [
    "Automatic installation is unavailable for this build",
    "This build wasn’t installed with automatic update support. You can keep using SunCode.",
  ],
  unconfigured: [
    "Updates are not configured for this build",
    "Update delivery is not enabled in this build. You can keep using SunCode.",
  ],
  idle: ["Check for a newer version", "Your installed versions are shown below."],
  checking: ["Checking for updates…", "Looking for updates for both components."],
  current: ["SunCode is up to date", "Both components are on the latest compatible version."],
  downloading: ["Downloading update…", "You can keep working while the download finishes."],
  verifying: ["Verifying update…", "The download will be checked before it can be installed."],
  ready: ["Your update is ready", "Restart SunCode to finish installing."],
  busy: [
    "Your update is ready",
    "Finish active turns and close running terminals before restarting.",
  ],
  "check-failed": ["Couldn’t check for updates", "Check your connection and try again."],
  "download-failed": [
    "The download was interrupted",
    "Your installed versions have not changed. Try downloading again.",
  ],
  "verification-failed": [
    "The update couldn’t be verified",
    "It won’t be installed. Download a fresh copy and try again.",
  ],
  "install-failed": [
    "The update couldn’t be installed",
    "Keep using your installed version and try again. If it still fails, reinstall SunCode.",
  ],
  restored: [
    "Your previous version was restored",
    "The update didn’t start correctly. You can keep using SunCode.",
  ],
  missing: [
    "Agent SDK is unavailable",
    "Download a compatible copy to repair SunCode, then restart.",
  ],
  installed: ["SunCode is up to date", "The update was installed successfully."],
};
