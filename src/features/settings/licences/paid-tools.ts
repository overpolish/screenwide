// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { t } from "../../../i18n/i18n";

import type { PaidTool } from "../../../bindings/PaidTool";

/** The paid tools, in the order the Licences pane lists them. */
export const PAID_TOOLS: readonly PaidTool[] = [
  "ruler",
  "annotate",
  "glide",
  "references",
];

/** A paid tool's name, as the Tools pane names its group. */
export const paidToolName = (tool: PaidTool) => {
  switch (tool) {
    case "annotate":
      return t("settings-section-annotate");
    case "glide":
      return t("settings-section-glide");
    case "references":
      return t("settings-licences-references");
    case "ruler":
      return t("settings-section-ruler");
  }
};
