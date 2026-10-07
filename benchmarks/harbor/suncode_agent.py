"""Harbor custom-agent adapter for the SunCode CLI.

The module is imported by Harbor only. SunCode production packages do not
depend on Python or Harbor.
"""

from __future__ import annotations

import base64
import os
import shlex
from pathlib import Path

from harbor.agents.capabilities import AgentCapabilities
from harbor.agents.installed.base import BaseInstalledAgent, with_prompt_template
from harbor.environments.base import BaseEnvironment
from harbor.models.agent.context import AgentContext

from .trajectory import write_trajectory


class SunCodeAgent(BaseInstalledAgent):
    capabilities = AgentCapabilities(atif=True)

    @staticmethod
    def name() -> str:
        return "suncode"

    def version(self) -> str | None:
        return "0.1.0"

    async def install(self, environment: BaseEnvironment) -> None:
        binary = os.environ.get("SUNCODE_AGENT_BINARY", "suncode")
        await self.exec_as_agent(environment, command=f"command -v {shlex.quote(binary)}")

    @with_prompt_template
    async def run(self, instruction: str, environment: BaseEnvironment, context: AgentContext) -> None:
        binary = os.environ.get("SUNCODE_AGENT_BINARY", "suncode")
        model = self.model_name.rsplit("/", 1)[-1] if self.model_name else None
        prompt = base64.b64encode(instruction.encode("utf-8")).decode("ascii")
        output_path = self.logs_dir / "suncode.jsonl"
        command = (
            f"printf %s {shlex.quote(prompt)} | base64 -d | "
            f"{shlex.quote(binary)} --output jsonl run . --stdin"
        )
        if model:
            command += f" --model {shlex.quote(model)}"
        result = await self.exec_as_agent(environment, command=command)
        output_path.write_text(result.stdout or "", encoding="utf-8")
        write_trajectory(
            result.stdout or "",
            str(self.logs_dir / "trajectory.json"),
            agent_name=self.name(),
            agent_version=self.version() or "unknown",
            model_name=self.model_name,
        )
