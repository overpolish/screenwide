// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AnnotateSettings } from "../../../bindings/AnnotateSettings";

/** Annotate settings for the islands' stories: the arrow in hand, in the
 * palette's yellow, and every tool's choices on one of the presets the
 * islands offer. */
export const PREVIEW_ANNOTATE_SETTINGS: AnnotateSettings = {
  defaultColor: "#ffcc00",
  defaultCounterAngle: 0,
  defaultCounterSize: 36,
  defaultHead: "end",
  defaultShape: "arrow",
  defaultWidth: 12,
  enabled: true,
  highlightHandDrawn: false,
  highlightManual: false,
  highlightTint: false,
  keepAnnotationsBetweenSessions: false,
  shapeHandDrawn: false,
  shapeRadius: 0,
  spotlightBlur: true,
  spotlightRadius: 25,
  spotlightSoftness: 10,
  toolbarPosition: null,
};

/** Colours of your own for the stories, after the palette. */
export const PREVIEW_SAVED_COLORS = ["#2ec4b6", "#8b5e34"];
