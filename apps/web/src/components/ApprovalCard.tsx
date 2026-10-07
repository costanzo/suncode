import { useWebStore } from "../store";

export function ApprovalCard() {
  const approval = useWebStore((state) => state.snapshot?.pendingApproval);
  const resolveApproval = useWebStore((state) => state.resolveApproval);
  if (!approval) return null;
  return (
    <div className="approval-card">
      <div>
        <strong>Approval required</strong>
        <small>{approval.summary} · policy decision</small>
      </div>
      <code>{approval.scope}</code>
      <p>{approval.detail ?? "SunCode is waiting for your policy decision."}</p>
      <div>
        <button
          className="primary-button"
          type="button"
          onClick={() => void resolveApproval("allow_once")}
        >
          Allow once
        </button>
        <button className="quiet-button" type="button" onClick={() => void resolveApproval("deny")}>
          Deny
        </button>
      </div>
    </div>
  );
}
