// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { FrameToolIcon } from "../../glyphs/frame-tool-icon";

import { FrameSection } from "./frame-section";

import type { DimensionsChange } from "../../tool-panels/dimensions/dimensions-field";
import type {
  Background,
  BackgroundPreset,
} from "../../tool-panels/frame/background/background";
import type { EditorIslandContent } from "../editor-island";

// The finished picture: how big it is, how round its corners are and what
// it sits on. Its size is the one place numbers are given, in the finished
// picture's pixels, as a frame drag moves them. Its corners are presets, for
// a screenshot only: a video's are never rounded. Its background is chosen
// from tiles, its colours set under them, or a picture of your own; then
// putting the size back as the capture arrived.

export type FrameIslandProps = {
  background: Background;
  /** In output pixels. */
  height: number;
  onBackgroundChange: (background: Background) => void;
  /** Puts the size back as the capture arrived. */
  onOriginalSize: () => void;
  /** Asks for a picture of your own, resolving to its path, or nothing. */
  onPickImage: () => Promise<string | null>;
  /** Forgets a kept background, by its id. */
  onRemovePreset: (presetId: string) => void;
  onSavePreset: (background: Background) => void;
  /** A changed side, or both while the aspect ratio is kept. */
  onSizeChange: (size: DimensionsChange) => void;
  /** The backgrounds that ship with the app. */
  presets: BackgroundPreset[];
  /** The backgrounds kept from before. */
  savedPresets: BackgroundPreset[];
  /** In output pixels. */
  width: number;
  /** Rounds the corners, by a percent of the picture's shorter side. */
  onRadiusChange?: (radius: number) => void;
  /** Percent of the picture's shorter side. A screenshot's corners can be
   * rounded; a video's never are, so a video leaves this out and the island
   * offers no corners. */
  radius?: number;
};

export function frameIsland(props: FrameIslandProps): EditorIslandContent {
  return {
    key: "frame",
    section: <FrameSection {...props} />,
    title: (
      <IslandRowTitle
        glyph={<FrameToolIcon aria-hidden className="size-icon" />}
        title={t("editor-tool-frame")}
      />
    ),
  };
}
