// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { FrameSizePreset } from "./frame/frame-sizes";
import type { PhoneView } from "../../../bindings/PhoneView";
import type { PlatformOverlay } from "../../../bindings/PlatformOverlay";
import type { FormatDifferences } from "../recording/formats/format-differences";
import type { EditScope } from "../recording/formats/format-scope";
import type { FormatSync } from "../recording/formats/format-sync";
import type { FormatVersion } from "../recording/formats/format-versions";
import type { LeftOut } from "../screenshot/formats/screenshot-show-in";

/** What the frame panel shows about the output canvas. */
export type ToolPanelFrame = {
  height: number;
  sourceHeight: number;
  sourceWidth: number;
  width: number;
  /** A recording's formats; left out for a screenshot. */
  formats?: ToolPanelFormats;
  /** How rounded the output canvas corners are, as a share of its shorter side. */
  radius?: number;
};

/** The shapes a recording is made in, one in hand, each with the formats
 * published in it. */
export type ToolPanelFormats = {
  /** The shape in hand; null while the recording is made in one shape. */
  activeId: string | null;
  /** The format in hand: a platform's template, or the shape's id for a
   * shape made for none. */
  formatId: string | null;
  /** The shape in hand's canvas, in output pixels. */
  height: number;
  list: {
    /** Its formats, in the order they were added; a shape made for no
     * platform has one, its own. `leavesOut` marks a screenshot's format
     * that leaves out a layer or an annotation the others show. */
    formats: { id: string; preset: string | null; leavesOut?: boolean }[];
    height: number;
    id: string;
    /** Whether its background is its own, changed while it was in hand,
     * rather than the main canvas's. */
    ownBackground: boolean;
    /** Whether it is framed its own way rather than following the main
     * canvas: its screen placed there, or a scene laid out for it. */
    ownWay: boolean;
    /** Whether its screen is placed its own way, moved while it was in
     * hand. */
    placed: boolean;
    width: number;
  }[];
  /** The app whose interface is laid over the picture. */
  overlay: PlatformOverlay | null;
  /** The phone a tall shape in hand is previewed on, kept while the
   * preview is off; none for a wide one or without a phone. */
  phone: PhoneView | null;
  /** The size template the format in hand was made from. */
  preset: string | null;
  /** Whether the phone and the app are shown over the picture. */
  preview: boolean;
  /** Where a layout changed in the shape in hand lands: the shared layout
   * in the main shape, the shape's own in any other. */
  scope: EditScope;
  width: number;
  /** What the format in hand does differently from the others, and what
   * it leaves out, for the strip's badges and ghosts. */
  differs?: FormatDifferences;
  /** What a screenshot's format in hand leaves out, which its picture
   * keeps but does not draw. */
  leftOut?: LeftOut;
  /** How much larger than on the main canvas a screenshot's format in hand
   * draws what its annotations draw (`sizeScaleOf`); one where unset. */
  sizeScale?: number;
  /** What a scene, annotation, song or sound effect, or the caption track
   * offers in its menu to set it apart by format, by id. */
  syncOf?: (id: string) => FormatSync | null;
  /** The templates a platform is added from: a screenshot's pictures;
   * a recording's video sizes where unset. */
  templates?: readonly FrameSizePreset[];
  /** The versions the recording comes out in, for the lines over the
   * strip's overview; none while every format comes out the same. */
  versions?: readonly FormatVersion[];
};
