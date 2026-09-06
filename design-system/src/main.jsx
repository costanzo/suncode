import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { DesignSystemApp } from "./app/DesignSystemApp.jsx";
import "./styles/foundation.css";
import "./styles/layout.css";
import "./styles/components.css";
import "./styles/review.css";
import "./core/pages/styles/index.css";
import "./projects/desktop/styles/shared.css";
import "./projects/desktop/styles/tokens.css";
import "./projects/desktop/styles/project-hub.css";
import "./projects/desktop/styles/settings.css";
import "./projects/desktop/styles/about.css";
import "./projects/desktop/styles/dialog-window.css";
import "./projects/desktop/workspace/styles/index.css";
import "./styles/browser.css";

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <DesignSystemApp />
  </StrictMode>,
);
