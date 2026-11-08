import { useId } from "react";
import { Radio } from "../../../../../components/universal/radio/index.js";

export function QuestionFields({
  questions,
  answers,
  onChange,
  disabled = false,
  expanded = false,
}) {
  const groupId = useId();

  function selectOption(index, label) {
    const answer = answers[index];
    const selected = questions[index].multiple
      ? answer.selected.includes(label)
        ? answer.selected.filter((value) => value !== label)
        : [...answer.selected, label]
      : [label];
    onChange(index, { selected, custom: questions[index].multiple ? answer.custom : "" });
  }

  function setCustom(index, custom) {
    const answer = answers[index];
    onChange(index, {
      selected: !questions[index].multiple && custom.trim() ? [] : answer.selected,
      custom,
    });
  }

  return (
    <div className="workspace-question-fields">
      {questions.map((question, index) => (
        <fieldset className="workspace-question-field" key={index} disabled={disabled}>
          <legend>
            <span className="workspace-question-topic">
              {index + 1} · {question.header}
            </span>
            <strong>{question.question}</strong>
          </legend>
          <p className="workspace-question-instruction">
            {question.multiple ? "Choose all that apply" : "Choose one answer"}
            {question.custom && (question.multiple ? " or add your own." : ", or write your own.")}
          </p>
          <div className="workspace-question-options">
            {question.options.map((option) => {
              const checked = answers[index].selected.includes(option.label);
              const controlProps = {
                name: `${groupId}-${index}`,
                value: option.label,
                checked,
                onChange: () => selectOption(index, option.label),
                className: "workspace-question-radio",
              };
              return (
                <label
                  key={option.label}
                  className={`workspace-question-option ${checked ? "is-selected" : ""}`}
                >
                  {question.multiple ? (
                    <input type="checkbox" {...controlProps} />
                  ) : (
                    <Radio {...controlProps} />
                  )}
                  <span>
                    <strong>{option.label}</strong>
                    <small>{option.description}</small>
                  </span>
                </label>
              );
            })}
          </div>
          {question.custom && (
            <textarea
              className="workspace-question-custom"
              value={answers[index].custom}
              rows={expanded ? 3 : 2}
              onChange={(event) => setCustom(index, event.target.value)}
              placeholder="Write your own answer"
              aria-label={`Custom answer for ${question.header}`}
            />
          )}
        </fieldset>
      ))}
    </div>
  );
}
