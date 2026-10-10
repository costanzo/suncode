import json
import unittest

try:
    from harbor.models.trajectories import Trajectory
except ImportError:
    Trajectory = None

from .trajectory import convert_jsonl


class TrajectoryTests(unittest.TestCase):
    def test_converts_messages_tools_and_final_result(self):
        output = "\n".join(
            [
                '{"type":"message.user","session_id":"s1","data":{"message":{"content":[{"type":"text","text":"Fix it"}]}}}',
                '{"type":"tool.requested","session_id":"s1","data":{"tool_call_id":"c1","name":"bash","arguments":{"command":"pytest"}}}',
                '{"type":"tool.result","session_id":"s1","data":{"tool_call_id":"c1","result":{"success":true}}}',
                '{"type":"run.result","session_id":null,"data":{"session_id":"s1","response":{"message":{"content":[{"type":"text","text":"Done"}]},"usage":{"input_tokens":2,"output_tokens":1,"total_tokens":3}}}}',
            ]
        )
        trajectory = convert_jsonl(output, model_name="openai/gpt-5.6-sol")
        self.assertEqual(trajectory["schema_version"], "ATIF-v1.7")
        self.assertEqual(trajectory["session_id"], "s1")
        self.assertEqual(trajectory["steps"][0]["source"], "user")
        self.assertEqual(trajectory["steps"][1]["tool_calls"][0]["function_name"], "bash")
        self.assertEqual(trajectory["steps"][1]["message"], "")
        self.assertEqual(json.loads(trajectory["steps"][1]["observation"]["results"][0]["content"]), {"success": True})
        self.assertEqual(trajectory["final_metrics"]["total_prompt_tokens"], 2)
        self.assertEqual(trajectory["final_metrics"]["total_completion_tokens"], 1)
        self.assertEqual(trajectory["final_metrics"]["extra"]["total_tokens"], 3)
        if Trajectory is not None:
            Trajectory.model_validate(trajectory)

    def test_assistant_and_requested_events_do_not_duplicate_tool_calls(self):
        output = "\n".join([
            '{"type":"message.assistant","session_id":"s1","data":{"message":{"content":[],"tool_calls":[{"call_id":"c1","name":"bash","arguments":{"command":"pytest"}}]}}}',
            '{"type":"tool.requested","session_id":"s1","data":{"tool_call_id":"c1","name":"bash","arguments":{"command":"pytest"}}}',
            '{"type":"tool.result","session_id":"s1","data":{"tool_call_id":"c1","result":{"success":true}}}',
            '{"type":"turn.completed","session_id":"s1","data":{"usage":{"input_tokens":2,"output_tokens":1,"total_tokens":3}}}',
        ])
        trajectory = convert_jsonl(output)
        self.assertEqual(len(trajectory["steps"]), 1)
        self.assertEqual(trajectory["steps"][0]["tool_calls"][0]["tool_call_id"], "c1")
        self.assertEqual(trajectory["final_metrics"]["total_prompt_tokens"], 2)
        if Trajectory is not None:
            Trajectory.model_validate(trajectory)

    def test_resume_report_supplies_session_identity_and_usage(self):
        trajectory = convert_jsonl('{"type":"session.resume.result","session_id":null,"data":{"session_id":"s1","response":{"message":{"content":[{"type":"text","text":"Done"}]},"usage":{"input_tokens":2,"output_tokens":1,"total_tokens":3}}}}')
        self.assertEqual(trajectory["session_id"], "s1")
        self.assertEqual(trajectory["steps"][0]["message"], "Done")
        self.assertEqual(trajectory["final_metrics"]["total_completion_tokens"], 1)
        if Trajectory is not None:
            Trajectory.model_validate(trajectory)


if __name__ == "__main__":
    unittest.main()
