// Conversation fixtures used by the desktop workspace specimens.

export const workspaceModelGroups = [
  { id: "openai", label: "OpenAI", models: ["gpt-5.6-sol", "gpt-5.5"] },
  { id: "claude", label: "Claude", models: ["claude-sonnet-5", "claude-opus-5"] },
  { id: "deepseek", label: "DeepSeek", models: ["deepseek-v4-flash", "deepseek-v4-pro"] },
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
