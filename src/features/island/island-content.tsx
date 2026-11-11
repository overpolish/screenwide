// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnnotateIsland } from "../annotate/island/annotate-island";
import { useAnnotateRows } from "../annotate/island/annotate-rows";
import { useAnnotateDrawing } from "../annotate/island/use-annotate-drawing";
import { useAnnotateEditing } from "../annotate/island/use-annotate-editing";
import { RecordingIslandControls } from "../recording-controls/island/recording-island-controls";
import { selectStatus, useRecordingStore } from "../recording-controls/store";
import { ReferenceDropIsland } from "../reference/drop/reference-drop-island";
import { useReferenceDropIsland } from "../reference/drop/use-reference-drop-island";
import { RulerIsland } from "../ruler/island/ruler-island";
import { useRulerIsland } from "../ruler/island/use-ruler-island";
import { QuickScreenshotIsland } from "../screenshots/island/quick-screenshot-island";
import { ScrollingCaptureIsland } from "../screenshots/island/scrolling-capture-island";
import { useQuickScreenshotIsland } from "../screenshots/island/use-quick-screenshot-island";
import { useScrollingCaptureIsland } from "../screenshots/island/use-scrolling-capture-island";
import { TextRecognitionIsland } from "../text-recognition/island/text-recognition-island";
import { useTextRecognitionIsland } from "../text-recognition/island/use-text-recognition-island";
import { UpdateIsland } from "../updates/island/update-island";
import { useUpdateIsland } from "../updates/island/use-update-island";

import { SystemKeysNote } from "./system-keys-note";
import { useIslandSwap } from "./use-island-swap";
import { useSystemKeysAnnouncement } from "./use-system-keys-announcement";

import type { IslandHost } from "./island-window";

/** What the island shows: a scrolling capture while one runs, as it takes
 * the screen over; the hint while a picture is dragged with Option held, as
 * letting go keeps it as a reference; Quick Screenshot's while its overlay
 * waits for an area, over the Ruler, which steps back for it; Text
 * Recognition's or the Ruler's tools while either is up, even during a
 * recording, as neither can be used without them; the annotate tools on
 * their own island while the overlay draws with no recording under way; a
 * new version while one is offered, which comes out only while the island
 * is free; and otherwise the recording's, with the annotate tools under its
 * row while the overlay draws. */
export function IslandContent({ island }: { island: IslandHost }) {
  const isDrawing = useAnnotateDrawing();
  const editing = useAnnotateEditing();
  const ruler = useRulerIsland();
  const scrolling = useScrollingCaptureIsland();
  const quickScreenshot = useQuickScreenshotIsland();
  const referenceDrop = useReferenceDropIsland();
  const textRecognition = useTextRecognitionIsland();
  const update = useUpdateIsland();
  const status = useRecordingStore(selectStatus);
  const annotate = isDrawing ? editing : null;
  const rows = useAnnotateRows(annotate);
  const swap = useIslandSwap(
    scrolling.isUp
      ? "scrollingCapture"
      : referenceDrop.isUp
        ? "referenceDrop"
        : quickScreenshot.isUp
          ? "quickScreenshot"
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
  // The first time one of the system's keys opens a tool, the island it
  // opens says so: Quick Screenshot's, Text Recognition's or the recording
  // controls before a recording.
  const sayer =
    swap.kind === "quickScreenshot" ||
    swap.kind === "textRecognition" ||
    (swap.kind === "recording" && status === "idle" && !annotate);
  const announcement = useSystemKeysAnnouncement(sayer && swap.host.isShown);
  const note =
    sayer && announcement.keys ? (
      <SystemKeysNote keys={announcement.keys} />
    ) : undefined;
  const host = {
    ...swap.host,
    onHidden: () => {
      swap.host.onHidden();
      announcement.onHidden();
    },
  };

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
  if (swap.kind === "referenceDrop") {
    return <ReferenceDropIsland {...swap.host} />;
  }
  if (swap.kind === "quickScreenshot" && quickScreenshot.destination) {
    return (
      <QuickScreenshotIsland
        {...host}
        destination={quickScreenshot.destination}
        note={note}
        onClose={quickScreenshot.onClose}
      />
    );
  }
  if (swap.kind === "textRecognition" && textRecognition.state) {
    return (
      <TextRecognitionIsland
        {...host}
        note={note}
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
    <RecordingIslandControls
      annotate={rows ?? undefined}
      island={host}
      note={note}
    />
  );
}
