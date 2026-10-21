// Conversation fixtures used by the desktop workspace specimens.

export const workspaceModelGroups = [
  { id: "openai", label: "OpenAI", models: ["gpt-5.6-sol", "gpt-5.5"] },
  { id: "claude", label: "Claude", models: ["claude-sonnet-5", "claude-opus-5"] },
  { id: "deepseek", label: "DeepSeek", models: ["deepseek-v4-flash", "deepseek-v4-pro"] },
  { id: "qwen", label: "Qwen", models: ["qwen3.8-max", "qwen3.8-flash"] },
  {
    id: "xai",
    label: "xAI",
    models: ["grok-4.7", "grok-4.6", "grok-4.5", "grok-4.3", "grok-build-0.1"],
  },
  {
    id: "minimax",
    label: "MiniMax",
    models: ["MiniMax-M3.1-Flash-Preview", "MiniMax-M3"],
  },
  { id: "meta", label: "Meta", models: ["muse-wonder-1.0", "muse-spark-1.3"] },
  { id: "mimo", label: "Xiaomi MiMo", models: ["mimo-v2.6-pro", "mimo-v2.6-flash"] },
];

export const runningConversationToolCalls = [
  {
    icon: "terminal",
    title: "Run mvn compile for the workspace shell specimen",
    state: "Running",
    tone: "running",
    request: "mvn -pl design-system compile",
    result: "Command still running",
    liveLabel: "Command output",
    liveOutput: [
      "[INFO] Scanning for projects...",
      "[INFO] ------------------------------------------------------------------------",
      "[INFO] Building design-system 0.0.0-review",
      "[INFO] --- frontend-maven-plugin:1.15.0:npm (npm install) @ design-system ---",
      "[INFO] added 214 packages in 4s",
      "[INFO] --- frontend-maven-plugin:1.15.0:npm (npm run build) @ design-system ---",
      "> design-system@0.0.0 build",
      "> vite build",
      "vite v7.1.3 building for production...",
      "transforming modules...",
      "rendering chunks...",
    ],
    error: "",
  },
  {
    icon: "files",
    title: "Updated workspace routes and modules",
    state: "Succeeded",
    tone: "success",
    request: "workspace route modules",
    result: "8 modules updated",
    error: "",
  },
];

export const longQueuedMessageText =
  "Please update the conversation layout using the attached reference. Keep the composer available while the build is running, preserve the existing image previews, and make the queue easier to scan without covering the assistant output.\n\nCheck the light and dark themes, keyboard navigation, and narrow windows. Long messages should remain readable when expanded, including all paragraphs and line breaks.\n\nFinally, verify the existing tests and summarize any remaining limitations. Do not discard or shorten this instruction when it is promoted to steering.";
