// Illustrative packages and hashes for the design preview, not a release catalog.
export const browserSources = {
  official: { label: "NuGet.org", url: "https://api.nuget.org/v3/index.json" },
  mirror: {
    label: "Huawei Cloud (China)",
    url: "https://repo.huaweicloud.com/repository/nuget/v3/index.json",
  },
};
export const browserTargets = {
  windows: { label: "Windows x64", package: "chromiumembeddedframework.runtime.win-x64" },
  macos: { label: "macOS ARM64", package: "cef.runtime.osx-arm64" },
  linux: { label: "Linux x64", package: "cef.runtime.linux-x64" },
};
export const browserVersion = "156.0.15";
export const browserExampleHash =
  "f8d5a013ab092cbc2b8b8ea34df78623aad076ff1d0839d1ef18f817806f72a3";
export const browserScenarios = [
  { value: "missing", label: "Not installed", phase: "missing" },
  { value: "downloading", label: "Downloading", phase: "downloading" },
  { value: "verifying", label: "SHA-256 verification", phase: "verifying" },
  { value: "installing", label: "Installing", phase: "installing" },
  { value: "ready", label: "Installed · stopped", phase: "ready", installed: true },
  {
    value: "running",
    label: "Browser running",
    phase: "ready",
    installed: true,
    runtime: "background",
  },
  {
    value: "user_controlled",
    label: "User control",
    phase: "ready",
    installed: true,
    runtime: "user_controlled",
  },
  {
    value: "disabled",
    label: "Browser Use disabled",
    phase: "ready",
    installed: true,
    enabled: false,
  },
  { value: "download_failed", label: "Download failed", phase: "download_failed" },
  { value: "hash_mismatch", label: "SHA-256 mismatch", phase: "hash_mismatch" },
  {
    value: "repair_failed",
    label: "Repair failed · previous version retained",
    phase: "hash_mismatch",
    installed: true,
  },
  { value: "source_missing", label: "Version missing from source", phase: "source_missing" },
  { value: "corrupt", label: "Installed files damaged", phase: "corrupt", installed: true },
  { value: "incompatible", label: "Incompatible version", phase: "incompatible", installed: true },
  {
    value: "restart_required",
    label: "Restart required",
    phase: "restart_required",
    installed: true,
  },
  { value: "unsupported", label: "Unsupported platform", phase: "unsupported" },
  { value: "no_project", label: "No project selected", phase: "ready", installed: true },
  {
    value: "runtime_failed",
    label: "Browser startup failed",
    phase: "ready",
    installed: true,
    runtime: "failed",
  },
];
export const browserPhaseLabels = {
  missing: "Not installed",
  downloading: "Downloading",
  verifying: "Verifying SHA-256",
  installing: "Installing",
  ready: "Ready",
  download_failed: "Download failed",
  hash_mismatch: "SHA-256 mismatch",
  source_missing: "Version unavailable",
  corrupt: "Repair needed",
  incompatible: "Version incompatible",
  restart_required: "Restart required",
  unsupported: "Unavailable on this platform",
};
export const browserPhaseMessages = {
  missing: "Install Chromium to use Browser Use and the built-in browser.",
  downloading: "Downloading the compatible Chromium package from your selected source.",
  verifying: "Checking the package against the SHA-256 in SunCode’s trusted component manifest.",
  installing: "Preparing verified files before activating the component.",
  ready: "Chromium is installed. It starts when you open the browser or use a browser tool.",
  download_failed: "The download was interrupted. Retry or choose another download source.",
  hash_mismatch:
    "The downloaded package failed SHA-256 verification and was discarded. Retry or choose another source.",
  source_missing:
    "This source does not have the required version. Choose another source and retry.",
  corrupt:
    "Required Chromium files are missing or damaged. Download a verified replacement to repair the component.",
  incompatible:
    "The installed Chromium version is incompatible with this SunCode desktop client. Install the compatible version.",
  restart_required: "The verified component is staged. Restart SunCode to activate it.",
  unsupported: "A compatible Chromium package is unavailable for this platform.",
};
