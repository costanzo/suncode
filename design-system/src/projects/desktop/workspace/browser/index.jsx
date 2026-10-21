import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { WindowSizeNote } from "../../WindowSizeNote.jsx";
import { BrowserPreviewPanel } from "../panels/browser-preview/BrowserPreviewPanel.jsx";

export function WorkspaceBrowserPage() {
  return (
    <>
      <PageHeader
        title="Built-in browser"
        description="The general Chromium surface for Browser Use tasks and user-controlled page review inside Workspace."
      />
      <WindowSizeNote width="980" height="700" minimumWidth="620" minimumHeight="520" />
      <Section
        id="workspace-built-in-browser"
        title="Browser surface"
        description="Browser Use context, local development pages, and ordinary browser navigation share one panel. Preview failures keep their error and retry path local; chat, model selection, and project switching remain available."
      >
        <div className="workspace-browser-standalone">
          <BrowserPreviewPanel />
        </div>
      </Section>
    </>
  );
}
