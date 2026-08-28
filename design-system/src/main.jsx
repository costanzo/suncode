import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { DesignSystemApp } from "../core/react/DesignSystemApp.jsx";
import "../core/styles/review.css";
import "../core/styles/browser.css";

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <DesignSystemApp />
  </StrictMode>,
);
