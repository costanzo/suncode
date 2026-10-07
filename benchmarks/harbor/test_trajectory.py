import unittest

from .trajectory import convert_jsonl


class TrajectoryTests(unittest.TestCase):
    def test_converts_messages_tools_and_final_result(self):
        output = "\n".join(
            [
                '{"type":"message.user","session_id":"s1","data":{"message":{"content":[{"type":"text","text":"Fix it"}]}}}',
                '{"type":"tool.requested","session_id":"s1","data":{"tool_call_id":"c1","name":"bash","arguments":{"command":"pytest"}}}',
                '{"type":"tool.result","session_id":"s1","data":{"tool_call_id":"c1","result":{"success":true}}}',
                '{"type":"run.result","session_id":"s1","data":{"response":{"message":{"content":[{"type":"text","text":"Done"}]},"usage":{"total_tokens":3}}}}',
            ]
        )
        trajectory = convert_jsonl(output, model_name="openai/gpt-5.6-sol")
        self.assertEqual(trajectory["schema_version"], "ATIF-v1.7")
        self.assertEqual(trajectory["session_id"], "s1")
        self.assertEqual(trajectory["steps"][0]["source"], "user")
        self.assertEqual(trajectory["steps"][1]["tool_calls"][0]["function_name"], "bash")
        self.assertEqual(trajectory["final_metrics"]["total_tokens"], 3)


if __name__ == "__main__":
    unittest.main()
