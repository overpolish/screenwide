// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PencilLine } from "lucide-react";
import { MouseEvent, PointerEvent } from "react";

import {
  boundsAnchor,
  pointerAnchor,
} from "../../../popup-panel/use-popup-menu";
import { annotationLaneLabel } from "../../annotations/annotation-kinds";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { SeekHandler } from "../../timeline/timeline-seek";
import { TimelineViewportState } from "../../timeline/timeline-viewport";
import { TimelineViewportContent } from "../../timeline/timeline-viewport-content";
import {
  TIMED_LANE_ROW_HEIGHT_PX,
  timedLaneFragmentBox,
} from "../../timeline/tracks/timed-lane-layout";
import { TimelineTrackHeader } from "../../timeline/tracks/timeline-track-header";

import { RecordingAnnotationClipEdges } from "./recording-annotation-clip-edges";
import { RecordingCameraPlacement } from "./recording-annotation-layers";
import {
  ANNOTATION_CLIP_MINIMUM_WIDTH_PX,
  recordingAnnotationRows,
} from "./recording-annotation-layout";
import {
  RecordingAnnotationPinBadge,
  RecordingAnnotationPinOverlay,
} from "./recording-annotation-pin-overlay";
import { withoutPinKeyframe } from "./recording-annotation-pins";
import { RecordingAnnotationClip } from "./recording-annotations";
import {
  AnnotationClipPinning,
  useAnnotationClipMenu,
} from "./use-annotation-clip-menu";
import { usePinKeyframeMenu } from "./use-pin-keyframe-menu";
import {
  togglesChoice,
  useRecordingAnnotationBand,
} from "./use-recording-annotation-band";
import { useRecordingAnnotationDrag } from "./use-recording-annotation-drag";
import { RecordingPinStatus } from "./use-recording-pin-status";

export function RecordingAnnotationLane({
  cameraPlacement = null,
  clips,
  edit,
  onChange,
  onClearSelection,
  onPreview,
  onSeek,
  onSelect,
  onSelectSwept,
  pinStatus,
  pinning,
  selectedIds,
  sourceDurationMs,
  viewport,
}: {
  clips: RecordingAnnotationClip[];
  edit: RecordingTimelineEdit;
  onChange: (clips: RecordingAnnotationClip[]) => void;
  onClearSelection: () => void;
  /** Choose the clip `id` alone, or with `toggle`, add or take it away. */
  onSelect: (id: string, toggle: boolean) => void;
  /** Choose the clips a band swept over, alone or added to the choice. */
  onSelectSwept: (ids: string[], additive: boolean) => void;
  selectedIds: ReadonlySet<string>;
  sourceDurationMs: number;
  viewport: TimelineViewportState;
  /** Where One video draws the screen and the camera at the playhead, which
   * a clip's menu moves its annotation between. */
  cameraPlacement?: RecordingCameraPlacement | null;
  onPreview?: (clips: RecordingAnnotationClip[] | null) => void;
  onSeek?: SeekHandler;
  /** How each pinned clip's path is coming along, by annotation id. */
  pinStatus?: ReadonlyMap<string, RecordingPinStatus>;
  /** What a clip's menu does to its pin. Without it, the lane offers no
   * pinning. */
  pinning?: AnnotationClipPinning;
}) {
  const { beginDrag, draft, laneRef, movedRef } = useRecordingAnnotationDrag({
    clips,
    edit,
    onCommit: onChange,
    onPreview,
    onSeek,
    onSelect: (id) => {
      onSelect(id, false);
    },
    sourceDurationMs,
    viewport,
  });
  const laidOut = recordingAnnotationRows(
    draft ?? clips,
    edit,
    sourceDurationMs,
  );
  const { band, pressLane } = useRecordingAnnotationBand({
    fragments: laidOut.fragments,
    laneRef,
    onClear: onClearSelection,
    onSweep: onSelectSwept,
    viewport,
  });
  // While a band is drawn, the lane shows the choice it will make.
  const isSelected = (id: string) =>
    band
      ? band.ids.has(id) || (band.additive && selectedIds.has(id))
      : selectedIds.has(id);
  const deleteKeyframe = (id: string, ms: number) => {
    onChange(
      clips.map((clip) =>
        clip.annotation.id === id && clip.pin
          ? { ...clip, pin: withoutPinKeyframe(clip.pin, ms) }
          : clip,
      ),
    );
  };
  const openKeyframeMenu = usePinKeyframeMenu(deleteKeyframe);
  const openClipMenu = useAnnotationClipMenu({
    cameraPlacement,
    clips,
    idPrefix: "annotation-clip:",
    onClipsChange: onChange,
    pinning,
    selectedIds,
  });
  return (
    <div className="flex items-center gap-section">
      <TimelineTrackHeader
        icon={<PencilLine />}
        isSelected={selectedIds.size > 0}
        label="Annotations"
      />
      <div
        className="relative min-w-0 grow overflow-hidden rounded-control bg-fill-tertiary"
        // Every press on a clip stops here, so what arrives is empty lane.
        onPointerDown={pressLane}
        ref={laneRef}
        style={{ height: laidOut.rowCount * TIMED_LANE_ROW_HEIGHT_PX }}
      >
        <TimelineViewportContent viewport={viewport}>
          {laidOut.fragments.map((fragment) => {
            const clip = fragment.item;
            const id = clip.annotation.id;
            // A counter is called by the number it shows; an arrow has no
            // name of its own, so it is called by its place in the lane.
            const label = annotationLaneLabel(
              clip.annotation,
              clips.findIndex((item) => item.annotation.id === id),
            );
            const selected = isSelected(id);
            const status = pinStatus?.get(id);
            const block = { edit, fragment, sourceDurationMs };
            const clickBody = (event: MouseEvent) => {
              event.stopPropagation();
              if (movedRef.current) {
                event.preventDefault();
                movedRef.current = false;
              } else onSelect(id, togglesChoice(event));
            };
            // A press with the toggle held is a click to come, never a drag.
            // One on a clip of several chosen carries them all.
            const pressBody = (event: PointerEvent) => {
              if (event.button !== 0) return;
              event.stopPropagation();
              if (togglesChoice(event)) return;
              beginDrag({
                clientX: event.clientX,
                clientY: event.clientY,
                edge: "body",
                id,
                ids:
                  selectedIds.size > 1 && selectedIds.has(id)
                    ? selectedIds
                    : undefined,
              });
            };
            return (
              <div
                className={`absolute overflow-hidden rounded-control text-footnote ${fragment.continuesPrevious ? "rounded-l-none" : ""} ${fragment.continuedByNext ? "rounded-r-none" : ""} ${selected ? "bg-primary-surface text-primary-fg" : "bg-fill-secondary text-content-fg"}`}
                key={fragment.fragmentId}
                onContextMenu={(event) => {
                  event.preventDefault();
                  void openClipMenu(
                    pointerAnchor(event.clientX, event.clientY),
                    clip,
                  );
                }}
                style={{
                  ...timedLaneFragmentBox(fragment.row),
                  left: `${String(fragment.outputStart * 100)}%`,
                  minWidth: ANNOTATION_CLIP_MINIMUM_WIDTH_PX,
                  width: `${String((fragment.outputEnd - fragment.outputStart) * 100)}%`,
                }}
              >
                <button
                  aria-label={label}
                  aria-pressed={selected}
                  className="h-full w-full truncate px-control-inset text-left focus-visible:outline-2 focus-visible:outline-primary"
                  onClick={clickBody}
                  // The menu key opens the right-click menu from the
                  // keyboard, hung off the block it acts on.
                  onKeyDown={(event) => {
                    const menuKey =
                      event.key === "ContextMenu" ||
                      (event.key === "F10" && event.shiftKey);
                    if (!menuKey) return;
                    event.preventDefault();
                    event.stopPropagation();
                    void openClipMenu(
                      boundsAnchor(event.currentTarget.getBoundingClientRect()),
                      clip,
                    );
                  }}
                  onPointerDown={pressBody}
                  type="button"
                >
                  {fragment.showLabel ? label : null}
                </button>
                {clip.pin ? (
                  <RecordingAnnotationPinOverlay
                    {...block}
                    clipEndMs={clip.endMs}
                    label={label}
                    onBodyClick={clickBody}
                    onBodyPointerDown={pressBody}
                    onDeleteKeyframe={(ms) => {
                      deleteKeyframe(id, ms);
                    }}
                    onKeyframeMenu={(point, ms, kind) => {
                      void openKeyframeMenu({
                        annotationId: id,
                        kind,
                        ms,
                        point,
                      });
                    }}
                    onSeek={onSeek}
                    pin={clip.pin}
                    status={status}
                  />
                ) : null}
                <RecordingAnnotationClipEdges
                  clip={clip}
                  clips={clips}
                  continuedByNext={fragment.continuedByNext}
                  continuesPrevious={fragment.continuesPrevious}
                  edit={edit}
                  label={label}
                  onChange={onChange}
                  onPress={(edge, event) => {
                    // Trimming one of several chosen keeps the choice.
                    if (!selectedIds.has(id)) onSelect(id, false);
                    beginDrag({
                      clientX: event.clientX,
                      clientY: event.clientY,
                      edge,
                      id,
                    });
                  }}
                  onSeek={onSeek}
                  sourceDurationMs={sourceDurationMs}
                />
                {clip.pin ? (
                  <RecordingAnnotationPinBadge
                    label={label}
                    selected={selected}
                    status={status}
                  />
                ) : null}
              </div>
            );
          })}
        </TimelineViewportContent>
        {band ? (
          <div
            aria-hidden
            className="pointer-events-none absolute bg-primary/15 inset-ring inset-ring-primary"
            style={{
              height: band.box.bottom - band.box.top,
              left: band.box.left,
              top: band.box.top,
              width: band.box.right - band.box.left,
            }}
          />
        ) : null}
      </div>
    </div>
  );
}
