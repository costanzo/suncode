import { useCallback, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Button } from "../../../../../components/universal/button/index.js";
import { Modal } from "../../../../../components/universal/modal/index.js";
import { Icon } from "../../../../../shared/Icon.jsx";
import { QuestionFields } from "./QuestionFields.jsx";

export function QuestionCard({ questions, outcome = "success", onResolved }) {
  const [answers, setAnswers] = useState(() => questions.map(() => ({ selected: [], custom: "" })));
  const [expanded, setExpanded] = useState(false);
  const [status, setStatus] = useState("idle");
  const [error, setError] = useState("");
  const timer = useRef(null);
  const failedOnce = useRef(false);
  const submitting = status === "submit" || status === "skip";
  const answered = answers.filter(
    (answer) => answer.selected.length || answer.custom.trim(),
  ).length;
  const close = useCallback(() => setExpanded(false), []);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  function updateAnswer(index, answer) {
    setAnswers((current) =>
      current.map((value, position) => (position === index ? answer : value)),
    );
  }

  function resolve(action) {
    if (timer.current) return;
    setStatus(action);
    setError("");
    timer.current = window.setTimeout(() => {
      timer.current = null;
      if (outcome === "fail-once" && !failedOnce.current) {
        failedOnce.current = true;
        setStatus("error");
        setError(
          action === "skip"
            ? "Couldn't skip the questions. Your answers are saved. Try again."
            : "Couldn't submit your answers. Your answers are saved. Try again.",
        );
        return;
      }
      setExpanded(false);
      onResolved(action === "skip" ? "Questions skipped" : "Answers submitted");
    }, 1200);
  }

  function actions(size) {
    return (
      <>
        <Button
          size={size}
          variant="primary"
          disabled={submitting || !answered}
          onClick={() => resolve("submit")}
        >
          {status === "submit" ? "Submitting…" : "Submit answers"}
        </Button>
        <Button size={size} variant="danger" disabled={submitting} onClick={() => resolve("skip")}>
          {status === "skip" ? "Skipping…" : "Skip"}
        </Button>
      </>
    );
  }

  return (
    <>
      <div className="workspace-question-card" aria-busy={submitting}>
        <header className="workspace-question-card-heading">
          <strong>Clarification needed</strong>
          <button
            type="button"
            className="workspace-icon-button"
            aria-label="Expand questions"
            title="Expand questions"
            onClick={() => setExpanded(true)}
          >
            <Icon name="expand" size={15} />
          </button>
        </header>
        <QuestionFields
          questions={questions}
          answers={answers}
          onChange={updateAnswer}
          disabled={submitting}
        />
        {error && !expanded && (
          <p className="workspace-question-error" role="alert">
            {error}
          </p>
        )}
        <footer className="workspace-question-actions">{actions("sm")}</footer>
      </div>
      {createPortal(
        <Modal
          open={expanded}
          title="Answer questions"
          description={`${questions.length} ${questions.length === 1 ? "question" : "questions"} · Current session`}
          onClose={close}
          closeOnBackdrop={false}
          className="workspace-question-window"
          actions={
            <div className="workspace-question-window-footer">
              {error && (
                <p className="workspace-question-error" role="alert">
                  {error}
                </p>
              )}
              <div className="workspace-question-window-actions">
                <span role="status" aria-live="polite">
                  {answered} of {questions.length} answered
                </span>
                <div>{actions("md")}</div>
              </div>
            </div>
          }
        >
          <div className="workspace-question-window-body" aria-busy={submitting}>
            <QuestionFields
              expanded
              questions={questions}
              answers={answers}
              onChange={updateAnswer}
              disabled={submitting}
            />
          </div>
        </Modal>,
        document.body,
      )}
    </>
  );
}
