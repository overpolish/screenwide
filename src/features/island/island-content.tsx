// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnnotateIsland } from "../annotate/island/annotate-island";
import { useAnnotateRows } from "../annotate/island/annotate-rows";
import { useAnnotateDrawing } from "../annotate/island/use-annotate-drawing";
import { useAnnotateEditing } from "../annotate/island/use-annotate-editing";
import { RecordingIslandControls } from "../recording-controls/island/recording-island-controls";
import { selectStatus, useRecordingStore } from "../recording-controls/store";

import { useIslandSwap } from "./use-island-swap";

import type { IslandHost } from "./island-window";

/** What the island shows: the annotate tools on their own island while the
 * overlay draws with no recording under way, and otherwise the recording's,
 * with the annotate tools under its row while the overlay draws. */
export function IslandContent({ island }: { island: IslandHost }) {
  const isDrawing = useAnnotateDrawing();
  const editing = useAnnotateEditing();
  const status = useRecordingStore(selectStatus);
  const annotate = isDrawing ? editing : null;
  const rows = useAnnotateRows(annotate);
  const swap = useIslandSwap(
    annotate && status === "idle" ? "annotate" : "recording",
    island,
  );

  // The annotate island keeps its tools while it tucks away after drawing
  // stops, so it leaves as it was rather than emptied.
  if (swap.kind === "annotate" && editing) {
    return (
      <AnnotateIsland
        {...editing}
        isShown={swap.host.isShown}
        notch={swap.host.notch}
        onHidden={swap.host.onHidden}
        onShapeChange={swap.host.onShapeChange}
      />
    );
  }
  return (
    <RecordingIslandControls annotate={rows ?? undefined} island={swap.host} />
  );
}
