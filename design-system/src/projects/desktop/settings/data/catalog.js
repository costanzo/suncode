export const providerCatalog = {
  deepseek: {
    label: "DeepSeek",
    endpoint: "https://api.deepseek.com",
    placeholder: "Paste DeepSeek API key",
    keyPreview: "sk-d••••••••7K2m",
    models: ["deepseek-v4-flash", "deepseek-v4-pro"],
  },
  zhipu: {
    label: "Zhipu GLM",
    endpoint: "https://open.bigmodel.cn/api/paas/v4",
    placeholder: "Paste Zhipu API key",
    keyPreview: "",
    models: ["glm-5.2", "glm-5.3"],
  },
  openai: {
    label: "OpenAI",
    endpoint: "https://api.openai.com/v1",
    placeholder: "Paste OpenAI API key",
    keyPreview: "sk-p••••••••9Xc4",
    models: ["gpt-5.5", "gpt-5.6-sol"],
  },
  kimi: {
    label: "Kimi",
    endpoint: "https://api.moonshot.ai/v1",
    placeholder: "Paste Kimi API key",
    keyPreview: "sk-k••••••••3FvP",
    models: ["kimi-k2.7-code", "kimi-k3"],
  },
  claude: {
    label: "Claude",
    endpoint: "https://api.anthropic.com/v1",
    placeholder: "Paste Anthropic API key",
    keyPreview: "sk-a••••••••6NwQ",
    models: ["claude-sonnet-5", "claude-opus-5"],
  },
  gemini: {
    label: "Gemini",
    endpoint: "https://generativelanguage.googleapis.com/v1beta/openai",
    placeholder: "Paste Gemini API key",
    keyPreview: "AIza••••••••2Lm8",
    models: ["gemini-3.5", "gemini-3.6-flash"],
  },
};

export const navItems = [
  { id: "defaults", label: "Defaults", icon: "foundation" },
  { id: "appearance", label: "Appearance", icon: "sun" },
  { id: "shortcuts", label: "Keyboard shortcuts", icon: "keyboard" },
  { id: "network", label: "Network", icon: "platform" },
  { id: "computer", label: "Computer use", icon: "computer" },
  { id: "browser", label: "Browser use", icon: "tool" },
  { id: "mcp", label: "MCP servers", icon: "server" },
  { id: "lsp", label: "Language servers", icon: "file-code" },
  { id: "logging", label: "Logging", icon: "assets" },
];

export const shortcutCatalog = [
  { action: "Open Settings", keys: ["⌘", ","], ariaLabel: "Command comma" },
  { action: "Toggle project navigation", keys: ["⌘", "1"], ariaLabel: "Command 1" },
  { action: "Toggle Git viewer", keys: ["⌘", "9"], ariaLabel: "Command 9" },
  { action: "Send message", keys: ["Enter"], ariaLabel: "Enter" },
  { action: "Submit session dialog", keys: ["Enter"], ariaLabel: "Enter" },
  { action: "Cancel current turn or close dialog", keys: ["Escape"], ariaLabel: "Escape" },
];
