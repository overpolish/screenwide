// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../i18n/i18n";

import { type GlideAction } from "./detection/glide-detection";
import { glideGridRows, type GlideRegion } from "./detection/glide-regions";

// Regions named in the app's language, the way `describeRegion` names them in
// English for debugging.

const columnLabel = ({ colSpan, colStart, gridCols }: GlideRegion) => {
  if (colSpan === gridCols) return null;
  const left = colStart === 0;
  if (gridCols === 2) {
    return left ? t("glide-region-left-half") : t("glide-region-right-half");
  }
  if (colSpan === 2) {
    return left
      ? t("glide-region-left-two-thirds")
      : t("glide-region-right-two-thirds");
  }
  if (colStart === 1) return t("glide-region-middle-third");
  return left ? t("glide-region-left-third") : t("glide-region-right-third");
};

const regionLabel = (region: GlideRegion) => {
  const columns = columnLabel(region);
  if (region.rowSpan === glideGridRows) {
    return columns ?? t("glide-region-full-screen");
  }
  const rows =
    region.rowStart === 0
      ? t("glide-region-top-half")
      : t("glide-region-bottom-half");
  return columns === null ? rows : t("glide-region-part", { columns, rows });
};

/** Names what a preview is showing, for its accessible name. */
export const describeDestination = (
  region: GlideRegion | null,
  pending: GlideAction | null,
) => {
  // An armed minimize wins: the region underneath is what an up step returns
  // to, not where the lift would place the window.
  if (pending === "minimize") return t("glide-minimize");
  return region
    ? t("glide-destination", { region: regionLabel(region) })
    : t("glide-no-destination");
};
