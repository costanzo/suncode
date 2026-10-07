import { useState } from "react";
import { useWebStore } from "../store";

export function QuestionCard() {
  const question = useWebStore((state) => state.snapshot?.pendingQuestion);
  const replyQuestion = useWebStore((state) => state.replyQuestion);
  const [answer, setAnswer] = useState("");
  if (!question) return null;
  return (
    <div className="approval-card question-card">
      <strong>{question.prompt}</strong>
      {question.options.map((option) => (
        <button
          className="quiet-button"
          key={option}
          type="button"
          onClick={() => void replyQuestion(option)}
        >
          {option}
        </button>
      ))}
      <input
        value={answer}
        onChange={(event) => setAnswer(event.target.value)}
        placeholder="Your answer"
      />
      <button
        className="primary-button"
        type="button"
        disabled={!answer.trim()}
        onClick={() => void replyQuestion(answer)}
      >
        Reply
      </button>
    </div>
  );
}
