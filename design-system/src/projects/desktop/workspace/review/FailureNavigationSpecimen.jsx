import { useEffect, useRef, useState } from "react";
import { Button } from "../../../../components/universal/button/index.js";
import { ReviewPanel } from "../panels/review/ReviewPanel.jsx";
import { ProviderTracePanel } from "../panels/provider-trace/ProviderTracePanel.jsx";

export function FailureNavigationSpecimen() {
  const [outcome, setOutcome] = useState("failed");
  const [drawer, setDrawer] = useState(null);
  const timer = useRef(null);
  const summary = useRef(null);
  useEffect(() => () => clearTimeout(timer.current), []);

  function reset(value = outcome) {
    clearTimeout(timer.current);
    setOutcome(value);
    setDrawer(null);
  }

  function inspect() {
    setDrawer("loading");
    timer.current = setTimeout(() => setDrawer(outcome), 650);
  }

  return (
    <div className="workspace-failure-specimen" ref={summary}>
      <div className="workspace-question-preview-controls">
        <label>
          Request details
          <select className="field" value={outcome} onChange={(event) => reset(event.target.value)}>
            <option value="failed">Available</option>
            <option value="loading">Loading</option>
            <option value="missing">No matching request</option>
            <option value="load-error">Load failure</option>
          </select>
        </label>
        <Button onClick={() => reset()}>Reset example</Button>
      </div>
      <ReviewPanel
        standalone
        state="failed"
        onInspectFailure={inspect}
        inspectingFailure={drawer === "loading"}
      />
      {drawer && (
        <ProviderTracePanel
          key={drawer}
          state={drawer}
          standalone
          onRetryDetails={() => {
            setOutcome("failed");
            setDrawer("failed");
          }}
          onClose={() => {
            clearTimeout(timer.current);
            setDrawer(null);
            summary.current?.querySelector(".workspace-failure-summary")?.focus();
          }}
        />
      )}
    </div>
  );
}
