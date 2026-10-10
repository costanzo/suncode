import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import AsyncMock

try:
    from harbor.agents.installed.base import NonZeroAgentExitCodeError
    from .suncode_agent import SunCodeAgent
except ModuleNotFoundError as exc:
    if not exc.name.startswith('harbor'):
        raise
    SunCodeAgent = None


@unittest.skipIf(SunCodeAgent is None, 'Harbor is not installed')
class AgentTests(unittest.IsolatedAsyncioTestCase):
    async def test_agent_environment_selects_binary_and_preserves_failed_output(self):
        with tempfile.TemporaryDirectory() as directory:
            logs = Path(directory)
            agent = SunCodeAgent(
                logs_dir=logs,
                model_name='deepseek/deepseek-v4-flash',
                extra_env={'SUNCODE_AGENT_BINARY': '/opt/suncode/suncode'},
            )
            output = '\n'.join([
                '{"type":"message.user","session_id":"s1","data":{"message":{"content":[{"type":"text","text":"Hello"}]}}}',
                '{"type":"error","data":{"code":"provider_unconfigured","message":"Missing key"}}',
            ])
            environment = SimpleNamespace(exec=AsyncMock(return_value=SimpleNamespace(
                stdout=output, stderr='diagnostic', return_code=3,
            )))
            with self.assertRaises(NonZeroAgentExitCodeError):
                await agent.run('Hello', environment, SimpleNamespace())
            command = environment.exec.call_args.kwargs['command']
            self.assertIn('/opt/suncode/suncode --output jsonl run', command)
            self.assertIn('--model deepseek-v4-flash', command)
            self.assertEqual((logs / 'suncode.jsonl').read_text(), output)
            self.assertEqual((logs / 'suncode.stderr.txt').read_text(), 'diagnostic')
            self.assertEqual(json.loads((logs / 'trajectory.json').read_text())['session_id'], 's1')

    async def test_install_uses_agent_environment_binary(self):
        with tempfile.TemporaryDirectory() as directory:
            agent = SunCodeAgent(logs_dir=Path(directory), extra_env={'SUNCODE_AGENT_BINARY': '/opt/suncode/suncode'})
            environment = SimpleNamespace(exec=AsyncMock(return_value=SimpleNamespace(stdout='', stderr='', return_code=0)))
            await agent.install(environment)
            self.assertIn('command -v /opt/suncode/suncode', environment.exec.call_args.kwargs['command'])


if __name__ == '__main__':
    unittest.main()
