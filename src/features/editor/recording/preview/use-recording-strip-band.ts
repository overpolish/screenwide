// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useEffect } from "react";

import { withAnnotationGlides } from "../../timeline/recording-strip/recording-strip-glides";
import { recordingStripLanes } from "../../timeline/recording-strip/recording-strip-lanes";
import { withAnnotationPins } from "../../timeline/recording-strip/recording-strip-pins";
import { recordedParts } from "../../timeline/recording-strip/recording-strip-time";
import { usePublishStripIsland } from "../../timeline/recording-strip/strip-island-channel";
import {
  BREATH_S,
  stripIslandItem,
} from "../../timeline/recording-strip/strip-island-item";
import { usePictureDrop } from "../../timeline/recording-strip/use-picture-drop";
import { useRecordingStrip } from "../../timeline/recording-strip/use-recording-strip";
import { useRecordingStripAudio } from "../../timeline/recording-strip/use-recording-strip-audio";
import { useRecordingStripServices } from "../../timeline/recording-strip/use-recording-strip-services";
import { type Playhead } from "../../timeline/scrub-playhead";
import { baseOf } from "../../timeline/strip/strip-part";
import { type StripSelection } from "../../timeline/strip/strip-selection";
import { type TimelineStripProps } from "../../timeline/strip/timeline-strip-props";

import {
  CAPTIONS_LAYER_ID,
  KEYBOARD_LAYER_ID,
  OverlayLayerId,
} from "./overlay-layers";
import { RecordingCanvasTool } from "./recording-canvas-tool";
import { useRecordingPreviewSelection } from "./use-recording-preview-selection";
import { useRecordingPreviewTransport } from "./use-recording-preview-transport";
import { useSoundTrackReplay } from "./use-sound-track-replay";

import type { ResolvedScrubPreviewProps } from "./recording-preview-props";

// The recording's strip under the picture, over the native player and the
// timeline edit: what it shows, what it is told, and what is chosen on it,
// published for the chosen item's island.

const aspectOf = (size: { height: number; width: number } | undefined) =>
  size && size.height > 0 ? size.width / size.height : 16 / 9;

export function useRecordingStripBand(
  props: ResolvedScrubPreviewProps,
  {
    changeCanvasTool,
    openOverlayTool,
    playhead,
    pressOverlayRef,
    selection,
    transport,
  }: {
    changeCanvasTool: (tool: RecordingCanvasTool) => void;
    openOverlayTool: (layerId: OverlayLayerId) => void;
    playhead: Playhead;
    pressOverlayRef: RefObject<(layerId: OverlayLayerId) => void>;
    selection: ReturnType<typeof useRecordingPreviewSelection>;
    transport: ReturnType<typeof useRecordingPreviewTransport>;
  },
) {
  const { artifactId, durationMs, keyboardEffects } = props;
  const { annotations, player, scenes, timelineBlade } = transport;
  const { edit } = timelineBlade.blade;
  const onEdit = (next: typeof edit) => {
    props.onRecordingTimelineEditChange?.(next);
  };
  const services = useRecordingStripServices({
    artifactId,
    captionSelection: selection.captionSelection,
    durationMs,
    edit,
    getPositionMs: player.getPositionMs,
    keyboard: selection.keyboardTimeline,
    onChoose: () => {
      annotations.clearSelection();
      scenes.selection.onClear();
    },
    onEdit,
    openOverlayTool,
    output: props.recordingOutput?.primary,
    pause: player.pause,
    playhead,
    pressOverlayRef,
    timelineDurationMs: timelineBlade.timelineDurationMs,
  });
  const pictureAspect = aspectOf(props.previewSourceDimensions.primary);
  const strip = useRecordingStrip({
    artifactId,
    durationMs,
    edit,
    hasKeyboardData: props.hasKeyboardData,
    isKeyboardShown: keyboardEffects.bake,
    isPlaying: player.isPlaying,
    isScreenShown: props.enabledVideoTracks.includes("primary"),
    moments: services.moments,
    onEdit,
    onPlayingChange: (isPlaying) => {
      if (isPlaying) player.play();
      else player.pause();
    },
    pictureAspect,
    playhead,
    seek: timelineBlade.seek,
    thumbnails: transport.timelineThumbnails.primary,
  });
  const lanes = recordingStripLanes({
    annotations,
    changeCanvasTool,
    map: strip.time.map,
    openCaptions: () => {
      openOverlayTool(CAPTIONS_LAYER_ID);
    },
    playhead,
    runs: strip.time.runs,
    scenes,
    seek: strip.seekTo,
    steps: strip.steps,
  });
  // An annotation or scene chosen anywhere, on the picture as on the
  // strip, is what is chosen: the strip lets go of its own choice.
  const laneChoice = lanes.chosen();
  const hasLaneChoice = laneChoice !== null;
  const { select } = strip;
  useEffect(() => {
    if (hasLaneChoice) select(null);
    // eslint-disable-next-line @eslint-react/exhaustive-deps -- `select` is remade each render; only a new lane choice lets go.
  }, [hasLaneChoice]);
  const stripSelection: StripSelection | null = laneChoice ?? strip.selection;
  const audio = useRecordingStripAudio({
    edit,
    playhead,
    select: strip.select,
    selection: strip.selection,
    steps: strip.steps,
    time: strip.time,
    timelineMs: timelineBlade.timelineDurationMs,
  });
  useSoundTrackReplay(edit, props.enabledStreamIndices, player);
  usePictureDrop({
    playhead,
    seek: timelineBlade.seek,
    timelineMs: timelineBlade.timelineDurationMs,
  });
  const stripProps: TimelineStripProps = {
    ...audio.strip,
    ...strip.strip({
      ...lanes.lanes,
      annotations: withAnnotationGlides(
        withAnnotationPins(
          lanes.lanes.annotations,
          annotations.clips,
          annotations.pinStatus,
        ),
        {
          clips: annotations.clips,
          partsOf: (span) =>
            recordedParts(strip.time.map, strip.time.runs, span),
          pictures: props.previewSourceDimensions,
          scenes: scenes.clips,
          videoMs: timelineBlade.timelineDurationMs,
        },
      ),
    }),
    onSelect: (next) => {
      strip.select(lanes.choose(next) ? null : next);
      // Shortcuts are styled all at once, so choosing some opens the
      // Keyboard island, which deletes them too.
      if (next?.kind === "shortcuts") openOverlayTool(KEYBOARD_LAYER_ID);
    },
    selection: stripSelection,
  };
  const chosenShortcuts =
    strip.selection?.kind === "shortcuts" ? strip.selection.ids : [];
  // What is chosen on the strip, for its island, and what the island's
  // asks do to it.
  const island =
    laneChoice === null
      ? (audio.island ??
        stripIslandItem({
          artifactId,
          beats: strip.beats,
          lulls: strip.lulls,
          map: strip.time.map,
          moments: services.moments,
          selection: strip.selection,
        }))
      : null;
  const chosen = {
    hasSongRoom: audio.hasSongRoom,
    item: island,
    shortcuts: chosenShortcuts.length,
    sounds: audio.sounds,
  };
  usePublishStripIsland("recording", chosen, {
    addSound: audio.addSound,
    changeAudioClip: audio.changeAudioClip,
    cut: strip.cutChosen,
    deleteShortcuts: () => {
      if (chosenShortcuts.length === 0) return;
      stripProps.onDeleteShortcuts(chosenShortcuts);
      strip.select(null);
    },
    deselect: () => {
      strip.select(null);
    },
    hearSounds: audio.hearSounds,
    playLull: (play) => {
      if (island?.kind === "lull")
        stripProps.onLullPlayChange(island.id, { kind: play });
    },
    trimLullToBreath: () => {
      if (island?.kind !== "lull") return;
      const lull = strip.lulls.lulls.find(({ id }) => id === baseOf(island.id));
      if (lull)
        stripProps.onLullTrim(
          island.id,
          Math.min(lull.found.start + BREATH_S, lull.found.end),
        );
    },
  });
  return {
    /** Whether songs or sound effects are laid under the recording, which
     * the Tracks island then switches. */
    hasSounds: audio.sounds.length > 0,
    island,
    select: strip.select,
    selection: strip.selection,
    strip: stripProps,
    transport: strip.transport,
  };
}
