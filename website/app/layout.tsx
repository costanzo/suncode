import type { Metadata } from "next";
import type { ReactNode } from "react";
import { SiteHeader } from "./components/SiteHeader";
import { SiteFooter } from "./components/SiteFooter";
import { themeBootstrap } from "../lib/theme";
import "./globals.css";

export const metadata: Metadata = {
  metadataBase: new URL("https://suncode.ai"),
  title: "SunCode | Keep the work in view",
  description:
    "A general-purpose coding agent with a native desktop, visible tool activity, scoped approvals, and undo for checkpointed file changes.",
  alternates: { canonical: "/" },
  openGraph: {
    title: "SunCode | Keep the work in view",
    description:
      "Understand, change, and review your projects with visible actions and decisions that stay yours.",
    url: "/",
    siteName: "SunCode",
    type: "website",
    images: [
      {
        url: "/og-image.png",
        width: 1200,
        height: 630,
        alt: "SunCode coding agent",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "SunCode | Keep the work in view",
    description:
      "A native coding agent with visible activity, scoped approvals, and checkpointed file undo.",
    images: ["/og-image.png"],
  },
  icons: {
    icon: [
      { url: "/favicon.ico", sizes: "any" },
      { url: "/favicon.svg", type: "image/svg+xml" },
    ],
    apple: { url: "/apple-touch-icon.png", sizes: "180x180" },
  },
};

export default function RootLayout({
  children,
}: Readonly<{ children: ReactNode }>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: themeBootstrap }} />
      </head>
      <body>
        <a className="skip-link" href="#main-content">
          Skip to content
        </a>
        <SiteHeader />
        {children}
        <SiteFooter />
      </body>
    </html>
  );
}
