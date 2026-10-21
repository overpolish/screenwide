// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

import type { RulerAction } from "../../../bindings/RulerAction";

// The keys that work while the ruler is showing, in the order they are
// learned: reading, copying, undoing, then placing.
export const rulerShortcuts = (): {
  action: RulerAction;
  label: string;
  description?: string;
}[] => [
  { action: "toggleCrosshair", label: t("settings-ruler-crosshair") },
  {
    action: "copyColour",
    description: t("settings-ruler-copy-colour-description"),
    label: t("settings-ruler-copy-colour"),
  },
  {
    action: "deleteMeasurement",
    description: t("settings-ruler-delete-description"),
    label: t("settings-ruler-delete"),
  },
  {
    action: "copyMeasurement",
    description: t("settings-ruler-copy-description"),
    label: t("settings-ruler-copy"),
  },
  { action: "undo", label: t("settings-ruler-undo") },
  { action: "redo", label: t("settings-ruler-redo") },
  {
    action: "stampHorizontal",
    description: t("settings-ruler-stamp-description"),
    label: t("settings-ruler-stamp-horizontal"),
  },
  {
    action: "stampVertical",
    description: t("settings-ruler-stamp-description"),
    label: t("settings-ruler-stamp-vertical"),
  },
  {
    action: "guideVertical",
    description: t("settings-ruler-guide-vertical-description"),
    label: t("settings-ruler-guide-vertical"),
  },
  {
    action: "guideHorizontal",
    description: t("settings-ruler-guide-horizontal-description"),
    label: t("settings-ruler-guide-horizontal"),
  },
  {
    action: "cycleTolerance",
    description: t("settings-ruler-tolerance-description"),
    label: t("settings-ruler-tolerance"),
  },
  {
    action: "measureRadius",
    description: t("settings-ruler-radius-description"),
    label: t("settings-ruler-radius"),
  },
  { action: "toggleCenterlines", label: t("settings-ruler-centerlines") },
];
