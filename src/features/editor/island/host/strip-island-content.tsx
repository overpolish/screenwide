// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { audioClipIsland } from "../audio-clip/audio-clip-island";
import { beatIsland } from "../beat/beat-island";
import { gapIsland } from "../gap/gap-island";
import { lullIsland } from "../lull/lull-island";
import { momentIsland } from "../moment/moment-island";
import { rangeIsland } from "../range/range-island";
import { stripGroupIsland } from "../strip-group/strip-group-island";

import { backgroundChoices } from "./frame-island-content";

import type { EditorIslandKind } from "./editor-island-kind";
import type { AudioClipPreviewState } from "./use-audio-clip-preview";
import type { MomentNoteState } from "./use-moment-note";
import type { BackgroundPreset } from "../../tools/frame/background/background";
import type { ToolPanelPatch } from "../../tools/tool-panel-patch";
import type { ToolPanelSnapshot } from "../../tools/tool-panel-store";
import type { ToolPanelStripItem } from "../../tools/tool-panel-strip";
import type { EditorIslandContent } from "../editor-island";

// The islands of what is chosen on the strip, each from what the editor
// publishes of the choice (`ToolPanelStripItem`): a beat and its cut, a gap
// with its length and background, a lull and how it plays, a moment with
// its voice note, a drawn range, its GIF and its cut, and several beats and
// lulls gathered with Shift and their cut. Every cut is the strip's own, as
// Delete makes it.

type Inputs = {
  audioClip: AudioClipPreviewState;
  /** The backgrounds a gap's island offers, as the Frame island does. */
  backgrounds: { presets: BackgroundPreset[]; snapshot: ToolPanelSnapshot };
  change: (values: ToolPanelPatch) => void;
  /** The island asked for, which the published choice must still be. */
  island: EditorIslandKind;
  item: ToolPanelStripItem | null | undefined;
  moment: MomentNoteState;
};

export function stripIslandContent({
  audioClip,
  backgrounds,
  change,
  island,
  item,
  moment,
}: Inputs): EditorIslandContent | null {
  if (item?.kind !== island) return null;
  const onCut = () => {
    change({ cutStripItem: true });
  };
  switch (item.kind) {
    case "audioClip":
      return audioClipIsland({
        ...item,
        ...audioClip,
        onChange: (audioClip) => {
          change({ audioClip });
        },
      });
    case "beat":
      return beatIsland({ beat: item.beat, id: item.id, onCut });
    case "gap": {
      const { frame } = backgrounds.snapshot;
      return gapIsland({
        ...backgroundChoices(change, backgrounds),
        aspect: frame ? frame.width / Math.max(frame.height, 1) : 16 / 9,
        background: item.background ?? backgrounds.snapshot.background,
        hasOwnBackground: item.background !== null,
        id: item.id,
        onBackgroundChange: (gapBackground) => {
          change({ gapBackground });
        },
        onClearBackground: () => {
          change({ gapBackground: null });
        },
        onRemove: onCut,
      });
    }
    case "lull":
      return lullIsland({
        id: item.id,
        onCut,
        onPlayChange: (lullPlay) => {
          change({ lullPlay });
        },
        onTrimToBreath: () => {
          change({ trimLullToBreath: true });
        },
        play: item.play,
      });
    case "moment":
      return momentIsland({
        color: item.color,
        id: item.id,
        name: item.name,
        note: moment.note,
        onCut,
        onDownloadModel: moment.onDownloadModel,
        onTogglePlay: moment.onTogglePlay,
      });
    case "range":
      return rangeIsland({
        id: item.id,
        length: (item.played.endMs - item.played.startMs) / 1000,
        onCut,
        onDeselect: () => {
          change({ deselectStripItem: true });
        },
      });
    case "stripGroup":
      return stripGroupIsland({ count: item.count, id: item.id, onCut });
  }
}
