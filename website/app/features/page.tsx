import type { Metadata } from "next";
import { FeatureCatalog } from "../components/FeatureCatalog";

export const metadata: Metadata = {
  title: "Features | SunCode",
  description:
    "Explore SunCode's complete feature guide: project sessions, approvals, file undo, providers, MCP, language servers, documents, CLI, SDK, and preview capabilities.",
  alternates: { canonical: "/features" },
  openGraph: {
    title: "Features | SunCode",
    description:
      "A complete guide to SunCode's capabilities and current availability.",
    url: "/features",
  },
};

export default function FeaturesPage() {
  return (
    <main id="main-content" className="features-page shell">
      <header className="features-heading">
        <h1>
          The whole toolkit.
          <br />
          <span>With a clear view of its limits.</span>
        </h1>
        <p>
          Explore everything SunCode brings to your project—from the everyday
          coding loop to optional integrations and developer tools.
        </p>
      </header>
      <FeatureCatalog />
    </main>
  );
}
