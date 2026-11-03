// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnnotateIsland } from "../annotate/island/annotate-island";
import { useAnnotateRows } from "../annotate/island/annotate-rows";
import { useAnnotateDrawing } from "../annotate/island/use-annotate-drawing";
import { useAnnotateEditing } from "../annotate/island/use-annotate-editing";
import { RecordingIslandControls } from "../recording-controls/island/recording-island-controls";
import { selectStatus, useRecordingStore } from "../recording-controls/store";
import { RulerIsland } from "../ruler/island/ruler-island";
import { useRulerIsland } from "../ruler/island/use-ruler-island";
import { ScrollingCaptureIsland } from "../screenshots/island/scrolling-capture-island";
import { useScrollingCaptureIsland } from "../screenshots/island/use-scrolling-capture-island";
import { TextRecognitionIsland } from "../text-recognition/island/text-recognition-island";
import { useTextRecognitionIsland } from "../text-recognition/island/use-text-recognition-island";
import { UpdateIsland } from "../updates/island/update-island";
import { useUpdateIsland } from "../updates/island/use-update-island";

import { useIslandSwap } from "./use-island-swap";

import type { IslandHost } from "./island-window";

/** What the island shows: a scrolling capture while one runs, as it takes
 * the screen over; Text Recognition's or the Ruler's tools while either is
 * up, even during a recording, as neither can be used without them; the
 * annotate tools on their own island while the overlay draws with no
 * recording under way; a new version while one is offered, which comes out
 * only while the island is free; and otherwise the recording's, with the
 * annotate tools under its row while the overlay draws. */
export function IslandContent({ island }: { island: IslandHost }) {
  const isDrawing = useAnnotateDrawing();
  const editing = useAnnotateEditing();
  const ruler = useRulerIsland();
  const scrolling = useScrollingCaptureIsland();
  const textRecognition = useTextRecognitionIsland();
  const update = useUpdateIsland();
  const status = useRecordingStore(selectStatus);
  const annotate = isDrawing ? editing : null;
  const rows = useAnnotateRows(annotate);
  const swap = useIslandSwap(
    scrolling.isUp
      ? "scrollingCapture"
      : textRecognition.isUp
        ? "textRecognition"
        : ruler.isUp
          ? "ruler"
          : status !== "idle"
            ? "recording"
            : annotate
              ? "annotate"
              : update.isUp
                ? "update"
                : "recording",
    island,
  );

  if (swap.kind === "update") {
    return <UpdateIsland {...swap.host} {...update.props} />;
  }
  // An island keeps its tools while it tucks away after its tool stops, so
  // it leaves as it was rather than emptied.
  if (swap.kind === "scrollingCapture" && scrolling.state) {
    return (
      <ScrollingCaptureIsland
        {...swap.host}
        onCancel={scrolling.onCancel}
        state={scrolling.state}
      />
    );
  }
  if (swap.kind === "textRecognition" && textRecognition.state) {
    return (
      <TextRecognitionIsland
        {...swap.host}
        onAction={textRecognition.onAction}
        onCopyQrCode={textRecognition.onCopyQrCode}
        onOpenQrCode={textRecognition.onOpenQrCode}
        state={textRecognition.state}
      />
    );
  }
  if (swap.kind === "ruler" && ruler.parts) {
    return <RulerIsland {...ruler.parts} {...swap.host} />;
  }
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
