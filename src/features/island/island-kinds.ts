// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { IslandClaim } from "../../bindings/IslandClaim";

/** The islands the window can show, one at a time. */
export type IslandKind =
  | "annotate"
  | "dictation"
  | "quickScreenshot"
  | "recording"
  | "referenceDrop"
  | "ruler"
  | "scrollingCapture"
  | "textRecognition"
  | "update";

/** Which island shows each claim on the island (`claims.rs`), when it is on
 * top. Every claim must be here, so a new one does not compile until the
 * page knows what to draw for it. Notes and the controls show on the
 * recording's island, and the copied shot on Quick Screenshot's. */
export const ISLAND_KINDS = {
  annotate: "annotate",
  controls: "recording",
  copiedScreenshot: "quickScreenshot",
  dictation: "dictation",
  notes: "recording",
  quickScreenshot: "quickScreenshot",
  recording: "recording",
  referenceDrop: "referenceDrop",
  ruler: "ruler",
  scrollingCapture: "scrollingCapture",
  textRecognition: "textRecognition",
  update: "update",
} as const satisfies Record<IslandClaim, IslandKind>;
