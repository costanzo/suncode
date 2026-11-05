import catalog from "./feature-catalog.json";

export type FeatureStatus = "implemented" | "preview" | "release" | "tooling";
export type Feature = {
  id: string;
  group: string;
  title: string;
  status: FeatureStatus;
  description: string;
  details: string[];
  note: string;
};

export const featureGroups = catalog.groups;
export const features = catalog.features as Feature[];
export const githubUrl = "https://github.com/costanzo/suncode";
export const sourceUrl = `${githubUrl}/tree/main`;
export const statusLabels: Record<FeatureStatus, string> = {
  implemented: "Implemented",
  preview: "Preview",
  release: "Release verification",
  tooling: "Developer tooling",
};
