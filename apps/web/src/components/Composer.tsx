import { useState } from "react";
import { useWebStore } from "../store";
import { Icon } from "./Icon";

const ATTACH_UNAVAILABLE = "Image attachments are not available in the web client yet";

export function Composer() {
  const sendMessage = useWebStore((state) => state.sendMessage);
  const [value, setValue] = useState("");
  const submit = async () => {
    if (!value.trim()) return;
    const submittedValue = value;
    setValue("");
    // On failure the store reports the error; restore the draft unless the user typed again.
    if (!(await sendMessage(submittedValue))) {
      setValue((current) => (current.trim() ? current : submittedValue));
    }
  };
  return (
    <div className="composer">
      {/* Kept in place to match the design-system composer; the contract has no upload yet. */}
      <span title={ATTACH_UNAVAILABLE}>
        <button
          className="composer-icon"
          type="button"
          aria-label="Attach image"
          aria-description={ATTACH_UNAVAILABLE}
          disabled
        >
          <Icon name="plus" />
        </button>
      </span>
      <textarea
        value={value}
        onChange={(event) => setValue(event.target.value)}
        onKeyDown={(event) => {
          const composing = event.nativeEvent.isComposing || event.nativeEvent.keyCode === 229;
          if (event.key === "Enter" && !event.shiftKey && !composing) {
            event.preventDefault();
            void submit();
          }
        }}
        placeholder="Message SunCode…"
        rows={1}
      />
      <button
        className="send-button"
        type="button"
        aria-label="Send message"
        onClick={() => void submit()}
      >
        <Icon name="send" />
      </button>
    </div>
  );
}
