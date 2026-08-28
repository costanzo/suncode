import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { ConversationPanel } from "../WorkspacePrimitives.jsx";

export function WorkspaceConversationPage() {
  return <><PageHeader title="Conversation" description="User messages, agent work, tool activity, final responses, and the turn composer." status="Phase 1" tone="implemented" /><Section id="conversation-panel" title="Active conversation"><ConversationPanel standalone /></Section></>;
}
