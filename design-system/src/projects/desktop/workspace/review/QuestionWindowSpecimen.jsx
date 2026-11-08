import { useState } from "react";
import { Button } from "../../../../components/universal/button/index.js";
import { singleQuestion, longQuestions, eightQuestions } from "../data/questions.js";
import { ReviewPanel } from "../panels/review/index.js";

const examples = { single: singleQuestion, long: longQuestions, eight: eightQuestions };

export function QuestionWindowSpecimen() {
  const [example, setExample] = useState("long");
  const [outcome, setOutcome] = useState("success");
  const [revision, setRevision] = useState(0);
  const [result, setResult] = useState("");

  function reset() {
    setResult("");
    setRevision((current) => current + 1);
  }

  return (
    <div className="workspace-question-specimen">
      <div className="workspace-question-preview-controls">
        <label>
          Example
          <select
            className="field"
            value={example}
            onChange={(event) => {
              setExample(event.target.value);
              reset();
            }}
          >
            <option value="single">One question</option>
            <option value="long">Long questions · 3</option>
            <option value="eight">Eight questions</option>
          </select>
        </label>
        <label>
          Submission
          <select
            className="field"
            value={outcome}
            onChange={(event) => {
              setOutcome(event.target.value);
              reset();
            }}
          >
            <option value="success">Success</option>
            <option value="fail-once">Fail once, then retry</option>
          </select>
        </label>
        <Button onClick={reset}>Reset example</Button>
        <span role="status">{result && `${result} · Agent resumed`}</span>
      </div>
      <ReviewPanel
        key={`${example}-${outcome}-${revision}`}
        standalone
        state="question"
        questions={examples[example]}
        questionOutcome={outcome}
        onQuestionResolved={setResult}
      />
    </div>
  );
}
