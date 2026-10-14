// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  CAPTIONS_LAYER_ID,
  OverlayLayerId,
} from "../recording/preview/overlay-layers";
import { RecordingKeyboardTimelineItem } from "../types";

import { RecordingTimelineEdit } from "./editing/recording-timeline-edit";
import { Playhead } from "./scrub-playhead";
import { layoutTimedLaneItems } from "./tracks/timed-lane-layout";

import type { TimelineItemSelection } from "./tracks/timeline-item-selection";
import type { Caption } from "../../../bindings/Caption";

/** The lane id of the shortcut showing at `ratio` of the edited video: of
 * those still on screen there, the one that began last, which the picture
 * draws as the newest. */
export const keyboardFragmentAt = ({
  edit,
  hiddenFragmentIds,
  hiddenItemIds,
  items,
  ratio,
  sourceDurationMs,
}: {
  edit: RecordingTimelineEdit;
  hiddenFragmentIds: ReadonlySet<string>;
  hiddenItemIds: ReadonlySet<number>;
  items: RecordingKeyboardTimelineItem[];
  ratio: number;
  sourceDurationMs: number;
}): string | null => {
  let showing: { fragmentId: string; outputStart: number } | null = null;
  for (const fragment of layoutTimedLaneItems({
    edit,
    items,
    sourceDurationMs,
  })) {
    if (
      fragment.outputStart > ratio ||
      fragment.outputEnd < ratio ||
      hiddenFragmentIds.has(fragment.fragmentId) ||
      hiddenItemIds.has(fragment.item.id)
    )
      continue;
    if (!showing || fragment.outputStart >= showing.outputStart)
      showing = fragment;
  }
  return showing?.fragmentId ?? null;
};

/** The lane id of the caption showing `atMs` into the edited video. */
export const captionAt = (captions: readonly Caption[], atMs: number) => {
  const caption = captions.find(
    ({ endMs, startMs }) => startMs <= atMs && atMs < endMs,
  );
  return caption ? String(caption.id) : null;
};

/**
 * What a press on an overlay in the picture does: chooses the shortcut or
 * caption showing at the playhead, as a click on it in its lane would, so
 * Delete takes it away, and opens the overlay's panel. Built beside the
 * lanes, which know what each overlay shows when.
 */
export const overlayPress =
  ({
    captionSelection,
    captions,
    edit,
    keyboard,
    onChoose,
    openOverlayTool,
    playhead,
    sourceDurationMs,
    timelineDurationMs,
  }: {
    captionSelection: TimelineItemSelection<string>;
    captions: readonly Caption[];
    edit: RecordingTimelineEdit;
    keyboard: {
      hiddenFragmentIds: ReadonlySet<string>;
      hiddenItemIds: ReadonlySet<number>;
      items: RecordingKeyboardTimelineItem[];
      selection: TimelineItemSelection<string>;
    };
    /** Lets go of what else is chosen, before an overlay is. */
    onChoose: () => void;
    openOverlayTool: (layerId: OverlayLayerId) => void;
    playhead: Playhead;
    sourceDurationMs: number;
    timelineDurationMs: number;
  }) =>
  (layerId: OverlayLayerId) => {
    const ratio = playhead.ratio();
    const [selection, id] =
      layerId === CAPTIONS_LAYER_ID
        ? [captionSelection, captionAt(captions, ratio * timelineDurationMs)]
        : [
            keyboard.selection,
            keyboardFragmentAt({ ...keyboard, edit, ratio, sourceDurationMs }),
          ];
    if (id !== null) {
      onChoose();
      selection.onSelect(id, false);
    }
    openOverlayTool(layerId);
  };
