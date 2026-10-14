// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Captions, Keyboard } from "lucide-react";
import { useMemo } from "react";

import { t } from "../../../../i18n/i18n";
import { RecordingKeyboardTimelineItem } from "../../types";
import { TimelineBladeController } from "../editing/timeline-blade";
import { Playhead } from "../scrub-playhead";
import { SeekHandler } from "../timeline-seek";
import { TimelineViewportState } from "../timeline-viewport";

import { timedLaneMinimumSpan } from "./timed-lane-layout";
import { TimelineItemLane } from "./timeline-item-lane";

import type { TimelineItemSelection } from "./timeline-item-selection";
import type { Caption } from "../../../../bindings/Caption";

/** The narrowest a shortcut badge is drawn, in pixels. */
const KEYBOARD_MINIMUM_ITEM_WIDTH_PX = 48;
/** The narrowest a caption is drawn, in pixels. Captions follow one another
 * and never overlap, so they keep one row and their true width; this only
 * keeps a one-word caption from vanishing zoomed out. */
const CAPTION_MINIMUM_ITEM_WIDTH_PX = 2;

/**
 * The shortcuts and captions lanes. A click chooses an item, Cmd or Ctrl
 * adds it to the choice, and a band drawn across empty lane chooses all it
 * touches. An item clicked alone also opens its panel and brings the
 * playhead to it, so the picture shows what was chosen.
 */
export function RecordingOverlayLanes({
  areaWidthPx,
  blade,
  captionSelection,
  captions,
  durationMs,
  hiddenKeyboardFragmentIds,
  hiddenKeyboardItemIds,
  keyboardItems,
  keyboardSelection,
  onOpenCaptionsPanel,
  onOpenKeyboardPanel,
  onSeek,
  playhead,
  sourceDurationMs,
  viewport,
}: {
  /** How wide the lanes are drawn, in pixels. */
  areaWidthPx: number;
  blade: TimelineBladeController;
  captionSelection: TimelineItemSelection<string>;
  captions: readonly Caption[];
  /** The edited video's length, in milliseconds, which captions are timed
   * on. */
  durationMs: number;
  keyboardItems: RecordingKeyboardTimelineItem[];
  keyboardSelection: TimelineItemSelection<string>;
  onSeek: SeekHandler;
  playhead: Playhead;
  sourceDurationMs: number;
  viewport: TimelineViewportState;
  hiddenKeyboardFragmentIds?: ReadonlySet<string>;
  hiddenKeyboardItemIds?: ReadonlySet<number>;
  onOpenCaptionsPanel?: () => void;
  onOpenKeyboardPanel?: () => void;
}) {
  // Overlapping shortcut badges stack into sublanes, so the lane is told the
  // span a badge needs; derived from the same shared stacking it renders from.
  const keyboardMinimumSpan = timedLaneMinimumSpan({
    contentWidthPx: areaWidthPx,
    minimumItemWidthPx: KEYBOARD_MINIMUM_ITEM_WIDTH_PX,
    zoom: viewport.zoom,
  });
  const captionItems = useMemo(
    () =>
      captions.map((caption) => ({
        endMs: caption.endMs,
        id: caption.id,
        label: caption.text.replace(/\n/gu, " "),
        startMs: caption.startMs,
      })),
    [captions],
  );
  // A deleted caption leaves the lane at once rather than when the captions
  // are next made without it.
  const hidden = blade.edit.captions?.hidden;
  const hiddenCaptionIds = useMemo(() => new Set(hidden), [hidden]);
  return (
    <>
      {keyboardItems.length > 0 ? (
        <TimelineItemLane
          edit={blade.edit}
          hiddenFragmentIds={hiddenKeyboardFragmentIds}
          hiddenItemIds={hiddenKeyboardItemIds}
          icon={<Keyboard />}
          items={keyboardItems}
          label={t("editor-timeline-shortcuts")}
          minimumItemWidthPx={KEYBOARD_MINIMUM_ITEM_WIDTH_PX}
          minimumSpan={keyboardMinimumSpan}
          onClearSelection={keyboardSelection.onClear}
          onSelect={(fragment, outputPosition, toggle) => {
            keyboardSelection.onSelect(fragment.fragmentId, toggle);
            if (toggle) return;
            onOpenKeyboardPanel?.();
            onSeek(outputPosition, "end");
          }}
          onSelectSwept={keyboardSelection.onSelectSwept}
          selectedFragmentIds={keyboardSelection.ids}
          sourceDurationMs={sourceDurationMs}
          viewport={viewport}
        />
      ) : null}
      {captionItems.length > 0 ? (
        <TimelineItemLane
          edit={blade.edit}
          hiddenItemIds={hiddenCaptionIds}
          icon={<Captions />}
          items={captionItems}
          label={t("editor-timeline-captions")}
          minimumItemWidthPx={CAPTION_MINIMUM_ITEM_WIDTH_PX}
          onClearSelection={captionSelection.onClear}
          onSelect={({ fragmentId, item }, _outputPosition, toggle) => {
            captionSelection.onSelect(fragmentId, toggle);
            if (toggle) return;
            onOpenCaptionsPanel?.();
            // The middle shows the caption fully on screen; a playhead
            // already in it is left where it was put.
            const start = item.startMs / durationMs;
            const end = item.endMs / durationMs;
            const at = playhead.ratio();
            if (at < start || at >= end) onSeek((start + end) / 2, "end");
          }}
          onSelectSwept={captionSelection.onSelectSwept}
          outputLengthMs={durationMs}
          selectedFragmentIds={captionSelection.ids}
          sourceDurationMs={sourceDurationMs}
          viewport={viewport}
        />
      ) : null}
    </>
  );
}
