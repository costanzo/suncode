import { useState } from "react";
import { currentTurnTodos } from "../data/review.js";
import { ReviewPanel } from "../panels/review/index.js";

const completedTodos = currentTurnTodos.map((todo) =>
  todo.status === "cancelled" ? todo : { ...todo, status: "completed", icon: "check" },
);

export function TodoLifecycleSpecimen() {
  const [stage, setStage] = useState("completed");
  const todos =
    stage === "next-turn" ? [] : stage === "completed" ? completedTodos : currentTurnTodos;

  return (
    <div className="workspace-question-specimen">
      <div className="workspace-question-preview-controls">
        <label>
          Turn state
          <select
            className="field"
            value={stage}
            onChange={(event) => setStage(event.target.value)}
          >
            <option value="running">Running</option>
            <option value="completed">Completed</option>
            <option value="failed">Failed</option>
            <option value="next-turn">Next turn started</option>
          </select>
        </label>
      </div>
      <ReviewPanel
        standalone
        state={stage === "next-turn" ? "running-no-changes" : stage}
        todos={todos}
      />
    </div>
  );
}
