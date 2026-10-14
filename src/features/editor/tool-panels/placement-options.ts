// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

import type { OverlaySize } from "../../../bindings/OverlaySize";
import type { Placement } from "../../../bindings/Placement";
import type { PositionGridItem } from "../../../components/base/position-grid/position-grid";

/** The nine places an overlay sits at, in reading order, as `PositionGrid`
 * lays them out. Captions and shortcuts offer the same ones. */
export const placementOptions = (): PositionGridItem<Placement>[] => [
  { id: "top-left", label: t("editor-panels-position-top-left") },
  { id: "top", label: t("editor-panels-position-top") },
  { id: "top-right", label: t("editor-panels-position-top-right") },
  { id: "left", label: t("editor-panels-position-left") },
  { id: "center", label: t("editor-panels-position-center") },
  { id: "right", label: t("editor-panels-position-right") },
  { id: "bottom-left", label: t("editor-panels-position-bottom-left") },
  { id: "bottom", label: t("editor-panels-position-bottom") },
  { id: "bottom-right", label: t("editor-panels-position-bottom-right") },
];

/** The sizes the shortcuts and the cursor are drawn at, smallest first: their
 * own size, then 150%, 200%, 300% and 500%. The twin of `OverlaySize` in
 * `screenwide_placement`, which says how large each one draws. */
export const OVERLAY_SIZES: OverlaySize[] = [
  "standard",
  "large",
  "extra-large",
  "huge",
  "giant",
];
