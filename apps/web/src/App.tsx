import { useEffect, useState } from "react";
import { Conversation } from "./components/Conversation";
import { Pairing } from "./components/Pairing";
import { ReviewPanel } from "./components/ReviewPanel";
import { Settings } from "./components/Settings";
import { Sidebar } from "./components/Sidebar";
import { useWebStore } from "./store";

export function App() {
  const hydrate = useWebStore((state) => state.hydrate);
  const hydrated = useWebStore((state) => state.hydrated);
  const reviewOpen = useWebStore((state) => state.reviewOpen);
  const [page, setPage] = useState<"workspace" | "settings" | "pairing">("workspace");
  useEffect(() => {
    void hydrate();
  }, [hydrate]);
  if (!hydrated) return <div className="loading-screen">Restoring secure browser session…</div>;
  if (page === "settings")
    return <Settings onBack={() => setPage("workspace")} onPair={() => setPage("pairing")} />;
  if (page === "pairing")
    return <Pairing onBack={() => setPage("settings")} onPaired={() => setPage("workspace")} />;
  return (
    <div className={`app-shell ${reviewOpen ? "review-open" : ""}`}>
      <Sidebar onSettings={() => setPage("settings")} />
      <Conversation />
      {reviewOpen && <ReviewPanel />}
    </div>
  );
}
