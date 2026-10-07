import type { MetadataRoute } from "next";

export default function sitemap(): MetadataRoute.Sitemap {
  return [{ url: "https://suncode.ai", lastModified: new Date("2026-10-07"), changeFrequency: "monthly", priority: 1 }];
}
