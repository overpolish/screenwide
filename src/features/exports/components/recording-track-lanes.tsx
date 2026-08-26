// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Camera, Keyboard, Monitor } from "lucide-react";
import {
  memo,
  PointerEvent as ReactPointerEvent,
  useRef,
  useState,
} from "react";

import { Checkbox } from "../../../components/base/checkbox/checkbox";
import {
  recordingAudioStreamIndex,
  recordingAudioTrackId,
  RecordingVideoTrackId,
} from "../types";

import { RecordingTrackContextMenu } from "./recording-track-context-menu";
import { RecordingTrackLanesProps } from "./recording-track-lanes-contract";
import { LayerContextMenuState } from "./screenshot-layer-context-menu";
import { ScrubAudioTracks } from "./scrub-audio-tracks";
import { TimelineAudioMeter } from "./timeline-audio-meter";
import { TimelineItemLane } from "./timeline-item-lane";
import { TimelineScrubberOverlay } from "./timeline-scrubber";
import { TimelineVideoClip } from "./timeline-video-clip";
import { TimelineHeader } from "./timeline-zoom-toolbar";
import { useTimelineNavigation } from "./use-timeline-navigation";

/** Memoized because pointer-rate canvas settings do not affect this subtree. */
export const RecordingTrackLanes = memo(function RecordingTrackLanes({
  adjustedKeyboardFragmentIds,
  audioTracks,
  blade,
  durationMs,
  enabledTracks,
  enabledVideoTracks,
  hiddenKeyboardFragmentIds,
  hiddenKeyboardItemIds,
  keyboardItems,
  keyboardSelection,
  layout,
  onEnabledTracksChange,
  onEnabledVideoTracksChange,
  onSeek,
  onSelectedTrackChange,
  onVideoTrackOrderChange,
  playhead,
  selectedTrack,
  sourceDurationMs,
  thumbnails,
  videoTrackOrder,
  volumes,
}: RecordingTrackLanesProps) {
  const [contextMenu, setContextMenu] =
    useState<LayerContextMenuState<RecordingVideoTrackId> | null>(null);
  const [drag, setDrag] = useState<{
    dropIndex: number;
    source: RecordingVideoTrackId;
  } | null>(null);
  const timeline = useTimelineNavigation(blade.edit.artifactId);
  const dragRef = useRef<{
    dropIndex: number;
    source: RecordingVideoTrackId;
    startY: number;
    started: boolean;
  } | null>(null);
  const rowElementsRef = useRef(
    new Map<RecordingVideoTrackId, HTMLDivElement>(),
  );
  const rowCount =
    layout.panes.length +
    audioTracks.length +
    (keyboardItems.length > 0 ? 1 : 0);
  const meterHeight = 30 + rowCount * 34;
  const videoRows = layout.panes
    .map((pane, index) => ({
      pane,
      trackId: index === 0 ? ("primary" as const) : ("camera" as const),
    }))
    .sort(
      (left, right) =>
        videoTrackOrder.indexOf(left.trackId) -
        videoTrackOrder.indexOf(right.trackId),
    );
  const applyOrder = (source: RecordingVideoTrackId, dropIndex: number) => {
    const sourceIndex = videoTrackOrder.indexOf(source);
    if (sourceIndex === -1) return;
    const next = videoTrackOrder.filter((track) => track !== source);
    const insertionIndex = Math.max(
      0,
      Math.min(
        next.length,
        dropIndex > sourceIndex ? dropIndex - 1 : dropIndex,
      ),
    );
    next.splice(insertionIndex, 0, source);
    if (next.some((track, index) => track !== videoTrackOrder[index]))
      onVideoTrackOrderChange?.(next);
  };
  const moveTrack = (
    track: RecordingVideoTrackId,
    direction: "backward" | "forward",
  ) => {
    setContextMenu(null);
    const index = videoTrackOrder.indexOf(track);
    const nextIndex = direction === "forward" ? index - 1 : index + 1;
    if (index === -1 || nextIndex < 0 || nextIndex >= videoTrackOrder.length)
      return;
    const next = [...videoTrackOrder];
    [next[index], next[nextIndex]] = [next[nextIndex], next[index]];
    onVideoTrackOrderChange?.(next);
  };
  const beginDrag =
    (source: RecordingVideoTrackId) =>
    (event: ReactPointerEvent<HTMLDivElement>) => {
      if (event.button !== 0 || videoRows.length < 2) return;
      if (
        event.target instanceof Element &&
        event.target.closest("button, input, [role='checkbox']")
      )
        return;
      dragRef.current = {
        dropIndex: videoTrackOrder.indexOf(source),
        source,
        startY: event.clientY,
        started: false,
      };
      event.currentTarget.setPointerCapture(event.pointerId);
    };
  const updateDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    const active = dragRef.current;
    if (!active) return;
    if (!active.started && Math.abs(event.clientY - active.startY) <= 4) return;
    active.started = true;
    event.preventDefault();
    let dropIndex = videoRows.length;
    for (let index = 0; index < videoRows.length; index += 1) {
      const row = rowElementsRef.current.get(videoRows[index].trackId);
      if (!row) continue;
      const bounds = row.getBoundingClientRect();
      if (event.clientY < bounds.top + bounds.height / 2) {
        dropIndex = index;
        break;
      }
    }
    active.dropIndex = dropIndex;
    setDrag({ dropIndex, source: active.source });
  };
  const finishDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    const active = dragRef.current;
    dragRef.current = null;
    if (active?.started) applyOrder(active.source, active.dropIndex);
    setDrag(null);
    if (event.currentTarget.hasPointerCapture(event.pointerId))
      event.currentTarget.releasePointerCapture(event.pointerId);
  };
  const cancelDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    dragRef.current = null;
    setDrag(null);
    if (event.currentTarget.hasPointerCapture(event.pointerId))
      event.currentTarget.releasePointerCapture(event.pointerId);
  };
  return (
    <section
      aria-label="Recording timeline"
      className="shrink-0 border-t border-muted/15 bg-content/55 pt-0.5 pr-3 pb-2 pl-3 [&_*]:outline-none! [&_*]:ring-0! [&_*]:ring-offset-0!"
      {...timeline.interactionProps}
    >
      <div className="flex items-stretch gap-2">
        <div className="relative flex min-w-0 grow flex-col gap-0.5">
          <TimelineHeader
            areaRef={timeline.areaRef}
            blade={blade}
            durationMs={durationMs}
            onFit={timeline.fit}
            onSeek={onSeek}
            onZoom={timeline.zoom}
            playhead={playhead}
            viewport={timeline.viewport}
          />
          {videoRows.map(({ pane, trackId }, rowIndex) => {
            const Icon = pane.kind === "camera" ? Camera : Monitor;
            const label = pane.kind === "camera" ? "Camera" : "Screen";
            const enabled = enabledVideoTracks.has(trackId);
            const mustRemainEnabled =
              enabled &&
              enabledVideoTracks.size === 1 &&
              enabledTracks.size === 0;
            return (
              <div
                className={`relative flex items-center gap-2 transition-opacity ${drag?.source === trackId ? "opacity-55" : ""}`}
                key={trackId}
                onContextMenu={(event) => {
                  event.preventDefault();
                  onSelectedTrackChange(trackId);
                  setContextMenu({
                    itemId: trackId,
                    x: Math.min(event.clientX, window.innerWidth - 196),
                    y: Math.min(event.clientY, window.innerHeight - 92),
                  });
                }}
                ref={(element) => {
                  if (element) rowElementsRef.current.set(trackId, element);
                  else rowElementsRef.current.delete(trackId);
                }}
              >
                {drag?.dropIndex === rowIndex ? (
                  <div className="pointer-events-none absolute -top-0.5 right-0 left-0 z-20 h-0.5 rounded bg-info" />
                ) : null}
                {rowIndex === videoRows.length - 1 &&
                drag?.dropIndex === videoRows.length ? (
                  <div className="pointer-events-none absolute -bottom-0.5 right-0 left-0 z-20 h-0.5 rounded bg-info" />
                ) : null}
                <div
                  className={`flex h-8 w-[calc(var(--recording-inspector-width,clamp(270px,23vw,300px))-1.25rem)] shrink-0 cursor-grab items-center gap-2 rounded px-2 text-xs font-medium text-content-fg transition-colors active:cursor-grabbing ${selectedTrack === trackId ? "bg-info/15" : ""}`}
                  onClick={() => {
                    onSelectedTrackChange(trackId);
                  }}
                  onPointerCancel={cancelDrag}
                  onPointerDown={beginDrag(trackId)}
                  onPointerMove={updateDrag}
                  onPointerUp={finishDrag}
                >
                  <Checkbox
                    aria-label={
                      mustRemainEnabled
                        ? `${label} must remain included`
                        : `${enabled ? "Exclude" : "Include"} ${label}`
                    }
                    isDisabled={mustRemainEnabled}
                    isSelected={enabled}
                    onChange={() => {
                      const next = new Set(enabledVideoTracks);
                      if (next.has(trackId)) {
                        if (next.size === 1 && enabledTracks.size === 0) return;
                        next.delete(trackId);
                      } else next.add(trackId);
                      onEnabledVideoTracksChange(next);
                    }}
                    size="xs"
                  />
                  <Icon className="shrink-0 text-muted" size={14} />
                  <span className="min-w-0 grow truncate">{label}</span>
                </div>
                <TimelineVideoClip
                  blade={blade}
                  enabled={enabled}
                  onSelect={onSelectedTrackChange}
                  selected={selectedTrack === trackId}
                  thumbnails={thumbnails[trackId]}
                  trackId={trackId}
                  viewport={timeline.viewport}
                />
              </div>
            );
          })}
          {audioTracks.length > 0 ? (
            <ScrubAudioTracks
              audioTracks={audioTracks}
              blade={blade}
              enabledTracks={enabledTracks}
              hasEnabledVideo={enabledVideoTracks.size > 0}
              onEnabledTracksChange={onEnabledTracksChange}
              onSelectTrack={(streamIndex) => {
                onSelectedTrackChange(recordingAudioTrackId(streamIndex));
              }}
              selectedTrack={recordingAudioStreamIndex(selectedTrack)}
              viewport={timeline.viewport}
              volumes={volumes}
            />
          ) : null}
          {keyboardItems.length > 0 ? (
            <TimelineItemLane
              edit={blade.edit}
              hiddenFragmentIds={hiddenKeyboardFragmentIds}
              hiddenItemIds={hiddenKeyboardItemIds}
              icon={<Keyboard size={14} />}
              items={keyboardItems}
              label="Shortcuts"
              minimumItemWidthPx={48}
              onClearSelection={keyboardSelection.onClear}
              onSelect={(fragment, outputPosition, toggle) => {
                blade.clearRangeSelection();
                blade.selectSegment(null);
                keyboardSelection.onSelect(fragment.fragmentId, toggle);
                onSeek(outputPosition, "end");
              }}
              selectedFragmentIds={keyboardSelection.ids}
              sourceDurationMs={sourceDurationMs}
              viewport={timeline.viewport}
              warningFragmentIds={adjustedKeyboardFragmentIds}
            />
          ) : null}
          <TimelineScrubberOverlay
            blade={blade}
            onSeek={onSeek}
            playhead={playhead}
            viewport={timeline.viewport}
          />
        </div>
        {audioTracks.length > 0 ? (
          <TimelineAudioMeter
            audioTracks={audioTracks}
            enabledTracks={enabledTracks}
            height={meterHeight}
            playhead={playhead}
            volumes={volumes}
          />
        ) : null}
      </div>
      <RecordingTrackContextMenu
        menu={contextMenu}
        onClose={() => {
          setContextMenu(null);
        }}
        onMove={moveTrack}
      />
    </section>
  );
});
