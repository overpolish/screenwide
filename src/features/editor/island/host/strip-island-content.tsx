// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { beatIsland } from "../beat/beat-island";
import { lullIsland } from "../lull/lull-island";
import { momentIsland } from "../moment/moment-island";
import { rangeIsland } from "../range/range-island";
import { stripGroupIsland } from "../strip-group/strip-group-island";

import type { ToolPanelPatch } from "../../tool-panels/tool-panel-patch";
import type { ToolPanelStripItem } from "../../tool-panels/tool-panel-strip";
import type { EditorIslandContent } from "../editor-island";
import type { EditorIslandKind } from "./editor-island-kind";
import type { MomentNoteState } from "./use-moment-note";

// The islands of what is chosen on the strip, each from what the editor
// publishes of the choice (`ToolPanelStripItem`): a beat and its cut, a
// lull and how it plays, a moment with its voice note, a drawn range and
// its cut, and several beats and lulls gathered with Shift and their cut.
// Every cut is the strip's own, as Delete makes it.

type Inputs = {
  change: (values: ToolPanelPatch) => void;
  /** The island asked for, which the published choice must still be. */
  island: EditorIslandKind;
  item: ToolPanelStripItem | null | undefined;
  moment: MomentNoteState;
};

export function stripIslandContent({
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
    case "beat":
      return beatIsland({ beat: item.beat, id: item.id, onCut });
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
      return rangeIsland({ id: item.id, onCut });
    case "stripGroup":
      return stripGroupIsland({ count: item.count, id: item.id, onCut });
  }
}
