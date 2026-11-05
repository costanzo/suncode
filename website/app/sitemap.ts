import type { MetadataRoute } from "next";

export default function sitemap(): MetadataRoute.Sitemap {
  return [
    {
      url: "https://suncode.ai",
      lastModified: new Date("2026-11-05"),
      changeFrequency: "monthly",
      priority: 1,
    },
    {
      url: "https://suncode.ai/features",
      lastModified: new Date("2026-11-05"),
      changeFrequency: "monthly",
      priority: 0.9,
    },
  ];
}
