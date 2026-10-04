// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PencilLine } from "lucide-react";
import { MouseEvent, PointerEvent } from "react";

import {
  boundsAnchor,
  pointerAnchor,
} from "../../../popup-panel/use-popup-menu";
import { annotationLaneLabel } from "../../annotations/annotation-kind-lookup";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { SeekHandler } from "../../timeline/timeline-seek";
import { TimelineViewportState } from "../../timeline/timeline-viewport";
import { TimelineViewportContent } from "../../timeline/timeline-viewport-content";
import {
  TIMED_LANE_ROW_HEIGHT_PX,
  timedLaneFragmentBox,
} from "../../timeline/tracks/timed-lane-layout";
import {
  togglesChoice,
  useTimelineLaneBand,
} from "../../timeline/tracks/timeline-lane-band";
import { TimelineLaneBandBox } from "../../timeline/tracks/timeline-lane-band-box";
import { TimelineTrackHeader } from "../../timeline/tracks/timeline-track-header";

import { RecordingAnnotationClipBody } from "./recording-annotation-clip-body";
import { RecordingAnnotationClipEdges } from "./recording-annotation-clip-edges";
import { RecordingCameraPlacement } from "./recording-annotation-layers";
import {
  ANNOTATION_CLIP_MINIMUM_WIDTH_PX,
  recordingAnnotationRows,
  sweptAnnotationClips,
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
  showsLayer = false,
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
  /** Whether each clip shows the picture it is drawn on: only a recording
   * with a camera has more than one. */
  showsLayer?: boolean;
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
  const { band, pressLane } = useTimelineLaneBand({
    laneRef,
    onClear: onClearSelection,
    onSweep: onSelectSwept,
    sweep: (box, laneWidthPx) =>
      sweptAnnotationClips(laidOut.fragments, box, { laneWidthPx, viewport }),
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
            // One on a clip of several chosen carries or trims them all.
            const ids =
              selectedIds.size > 1 && selectedIds.has(id)
                ? selectedIds
                : new Set([id]);
            const pressBody = (event: PointerEvent) => {
              if (event.button !== 0) return;
              event.stopPropagation();
              if (togglesChoice(event)) return;
              beginDrag({
                clientX: event.clientX,
                clientY: event.clientY,
                edge: "body",
                id,
                ids,
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
                <RecordingAnnotationClipBody
                  label={label}
                  onClick={clickBody}
                  onMenu={(bounds) => {
                    void openClipMenu(boundsAnchor(bounds), clip);
                  }}
                  onPointerDown={pressBody}
                  selected={selected}
                  showsLabel={fragment.showLabel}
                  showsLayer={showsLayer}
                  trackId={clip.trackId}
                />
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
                  ids={ids}
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
                      ids,
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
        {band ? <TimelineLaneBandBox box={band.box} /> : null}
      </div>
    </div>
  );
}
