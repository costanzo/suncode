# Requirement

ProjectHub-to-Workspace navigation must not wait for slow MCP server startup. MCP connections begin after Workspace opens, run in the Rust agent background, and expose progress to the footer. Chat and built-in tools remain usable while some MCP servers are connecting or failed.
