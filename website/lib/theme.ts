export type Theme = "light" | "dark";
export const themeStorageKey = "suncode-theme";

// Runs in the document head before content paints, independently of hydration.
export const themeBootstrap = `(function(){
  var preference;
  try { preference = localStorage.getItem("${themeStorageKey}"); } catch (_) {}
  var theme = preference === "light" || preference === "dark"
    ? preference
    : (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
  document.documentElement.setAttribute("data-theme", theme);
})();`;
