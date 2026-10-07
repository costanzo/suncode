import type { Metadata } from "next";
import type { ReactNode } from "react";
import "./globals.css";

export const metadata: Metadata = {
  metadataBase: new URL("https://suncode.ai"),
  title: {
    default: "SunCode | A coding agent you can review",
    template: "%s | SunCode",
  },
  description:
    "SunCode is a local-first coding agent for understanding, changing, reviewing, and maintaining software projects with visible, scoped, reversible machine access.",
  alternates: { canonical: "https://suncode.ai" },
  openGraph: {
    title: "SunCode | A coding agent you can review",
    description:
      "A general-purpose coding agent with visible tool activity, scoped approvals, and turn-level undo.",
    url: "https://suncode.ai",
    siteName: "SunCode",
    type: "website",
    images: [{ url: "/og-image.svg", width: 1200, height: 630, alt: "SunCode coding agent" }],
  },
  twitter: {
    card: "summary_large_image",
    title: "SunCode | A coding agent you can review",
    description: "A general-purpose coding agent with visible, scoped, reversible machine access.",
    images: ["/og-image.svg"],
  },
  icons: { icon: "/suncode-logo.svg" },
};

export default function RootLayout({ children }: Readonly<{ children: ReactNode }>) {
  const designContract = `<!-- THESIS: SunCode makes agentic coding feel like a reviewable workbench, not a magic box. OWN-WORLD: graphite surfaces, hairline rules, mint status lamps, and compact code data. STORY: developers see the agent inspect, ask, act, and leave a reversible trail. FIRST VIEWPORT: brand and promise sit left while a live-looking session slice proves the mechanism on the right. FORM: Persuade, product-workbench landing page, seed key quiet-control-desk. FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, and DESIGN.md -->`;

  return (
    <html lang="en">
      <body>
        <script
          id="design-contract"
          dangerouslySetInnerHTML={{
            __html: `document.body.insertAdjacentHTML("afterbegin", ${JSON.stringify(designContract)});`,
          }}
        />
        {children}
      </body>
    </html>
  );
}
