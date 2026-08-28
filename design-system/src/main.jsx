import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { DesignSystemApp } from "./app/DesignSystemApp.jsx";
import "./styles/review.css";
import "./styles/browser.css";

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <DesignSystemApp />
  </StrictMode>,
);
