// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnnotationStyle } from "../annotations";

import type { AnnotationKind } from "../../../components/shared/annotation-style/types";

/**
 * What the selection panel shows for a placed layer: the selected layer, its
 * size and position in output pixels, and the source it was captured at, which
 * is what a reset puts it back to.
 */
export type ToolPanelLayerSelection = {
  /** Whether this layer casts a shadow onto the canvas behind it. */
  dropShadow: boolean;
  height: number;
  /** How far the padded frame runs past the layer's own picture, in output
   * pixels, on each side. */
  inset: number;
  /** As far as the padding may be taken: the layer's shorter side. */
  insetMaximum: number;
  /** Which of the workspace's layers this is, for wording that fits it. */
  kind: "camera" | "layer" | "primary";
  label: string;
  /** How rounded the layer's corners are, as a share of its shorter side,
   * 0 to 50. The same number the corner drag in the preview sets. */
  radius: number;
  sourceHeight: number;
  sourceWidth: number;
  width: number;
  x: number;
  y: number;
  /** Camera only: whether baking it in is on the table at all. Baking draws
   * the camera into the screen's picture, so it needs both tracks kept. */
  canBake?: boolean;
  /** Camera only: whether it is drawn into the screen's picture rather than
   * carried as a track of its own. */
  isBaked?: boolean;
};

/**
 * What the selection panel shows for the keyboard shortcut on screen.
 *
 * A shortcut is drawn rather than placed: it has no source pixels, no corners
 * and no pad, so it is sized as a share of its natural size and positioned by
 * its centre, both in percent - the very numbers the drag on it sets.
 */
export type ToolPanelShortcutSelection = {
  kind: "shortcut";
  label: string;
  /** As big as this recording's widest shortcut may be drawn. */
  maximumSizePercent: number;
  minimumSizePercent: number;
  /** The shortcut's centre, as a share of the output canvas. */
  positionXPercent: number;
  positionYPercent: number;
  sizePercent: number;
};

/**
 * What the selection panel shows for a recorded audio track.
 *
 * An audio track is neither placed nor drawn: it is heard, so the only thing
 * there is to set for it is how loud it is played back, in decibels against
 * the level it was recorded at.
 */
export type ToolPanelAudioSelection = {
  /** How much the track is lifted or lowered, 0 being the recorded level. */
  decibels: number;
  kind: "audio";
  /** The track's own name: "Microphone", "System audio", or "Audio" where the
   * recording did not say. */
  label: string;
};

export type ToolPanelSelection =
  | ToolPanelAudioSelection
  | ToolPanelLayerSelection
  | ToolPanelShortcutSelection;

/**
 * What the annotation panel shows: the annotation the preview has in hand, and
 * the dress it is drawn in.
 *
 * An annotation is neither placed nor sized in output pixels - it is drawn, in
 * the source's own space, by the compositor - so there is nothing here of where
 * it sits: the picture itself is where it is moved, and the panel only ever
 * says what it looks like. `kind` is what the panel offers controls for: an
 * arrow has heads and a stroke, a counter a disc.
 */
export type ToolPanelAnnotation = {
  /** Whether this annotation draws itself in at the start of its clip and
   * undraws at the end. A screenshot has no clip to animate over, so the panel
   * shows it only in the recording editor. */
  animated: boolean;
  id: string;
  kind: AnnotationKind;
  style: AnnotationStyle;
  /** A counter's aim, in radians clockwise from east. Absent on an arrow,
   * which is aimed by its own two ends. */
  angle?: number;
};
