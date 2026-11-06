// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import type { AddKeyOutcome } from "../../../bindings/AddKeyOutcome";
import type { Licences } from "../../../bindings/Licences";
import type { PaidTool } from "../../../bindings/PaidTool";
import type { ToolAccess } from "../../../bindings/ToolAccess";

const DAY = 86_400;

/** Where the preview's licences start: before any paid tool is used, in the
 * trial, after it, or with every tool's key added. */
export type LicencesPreviewSeed = "unused" | "trial" | "expired" | "licensed";

const isSeed = (value: unknown): value is LicencesPreviewSeed =>
  value === "unused" ||
  value === "trial" ||
  value === "expired" ||
  value === "licensed";

export const licencesSeed = (value: unknown): LicencesPreviewSeed =>
  isSeed(value) ? value : "trial";

const TOOLS: PaidTool[] = ["ruler", "annotate", "glide", "references"];
const now = () => Math.floor(Date.now() / 1000);

// A key the preview pretends to know, by the tool it unlocks: any key that
// ends in a tool's name, as `DEMO-RULER`, unlocks that tool.
const toolFor = (key: string) =>
  TOOLS.find((tool) => key.toLowerCase().endsWith(tool));

/** The Licences pane's settings API, kept in memory for stories. */
export function licencesPreviewApi(seed: LicencesPreviewSeed) {
  const base: ToolAccess =
    seed === "unused"
      ? { kind: "trialUnused" }
      : seed === "expired"
        ? { kind: "expired" }
        : { endsAt: now() + 9 * DAY, kind: "trial" };
  let keys: Licences["keys"] =
    seed === "licensed"
      ? TOOLS.map((tool) => ({
          hint: tool.slice(0, 2).toUpperCase() + "7QK",
          id: tool,
          tools: [tool],
        }))
      : [];
  const licences = (): Licences => ({
    keys,
    tools: TOOLS.map((tool) => ({
      access: keys.some((held) => held.tools.includes(tool))
        ? { kind: "licensed" }
        : base,
      tool,
    })),
    trialDays: 14,
  });
  return {
    addLicenceKey: (key: string): Promise<AddKeyOutcome> => {
      if (key.toLowerCase().includes("offline"))
        return Promise.resolve({ kind: "unavailable" });
      if (key.toLowerCase().includes("refund"))
        return Promise.resolve({ kind: "disabled" });
      const tool = toolFor(key);
      if (!tool) return Promise.resolve({ kind: "invalidKey" });
      keys = [
        ...keys.filter((held) => held.id !== key),
        { hint: key.slice(-5), id: key, tools: [tool] },
      ];
      return Promise.resolve({ kind: "added", tools: [tool] });
    },
    getLicences: () => Promise.resolve(licences()),
    listenToLicences: () => Promise.resolve(() => undefined),
    listenToLicencesOpen: () => Promise.resolve(() => undefined),
    openLicenceCheckout: () => Promise.resolve(null),
    removeLicenceKey: (id: string) => {
      keys = keys.filter((held) => held.id !== id);
      return Promise.resolve(licences());
    },
  };
}
