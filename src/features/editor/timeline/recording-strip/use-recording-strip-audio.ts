// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { useEditorWindowShortcuts } from "../../shortcuts/use-editor-window-shortcuts";
import {
  type AudioClipKind,
  loopedClip,
  movedClip,
  type RecordingAudioClip,
  startedFrom,
  trimBounds,
  trimmedClip,
} from "../../sound-track/audio-clips";
import {
  useClipSounds,
  useSoundLibrary,
} from "../../sound-track/sound-library";
import { type TimelineAudioClip } from "../audio-lane/audio-stack";
import { type RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { type Playhead } from "../scrub-playhead";
import { baseOf, partId } from "../strip/strip-part";
import { type StripSelection } from "../strip/strip-selection";
import { type TimelineStripProps } from "../strip/timeline-strip-props";
import { type Span, type TimeMap } from "../timeline-scale";

import { audioClipWriter } from "./audio-clip-writer";
import { playedMs, recordedParts, retimedParts } from "./recording-strip-time";
import { type RecordingStripSteps } from "./use-recording-strip-edits";
import { useSoundAdding } from "./use-sound-adding";

import type { AudioClipChange } from "../../island/audio-clip/audio-clip-island";
import type { ToolPanelStripItem } from "../../tools/tool-panel-strip";

// The recording's songs and sound effects on its strip: drawn from the
// edit where they play, in parts either side of a jump as annotations are,
// moved, trimmed and restacked there, added as `useSoundAdding` adds them,
// and changed from the island of the one chosen. Each change is one step of
// the edit's history.

export function useRecordingStripAudio({
  edit,
  playhead,
  select,
  selection,
  steps,
  time,
  timelineMs,
}: {
  edit: RecordingTimelineEdit;
  playhead: Playhead;
  select: (next: StripSelection | null) => void;
  selection: StripSelection | null;
  steps: RecordingStripSteps;
  /** How the strip reads the edit's time. */
  time: { map: TimeMap; runs: Span[] };
  /** How long the edited video plays, in ms. */
  timelineMs: number;
}) {
  const clips = edit.audioClips ?? [];
  const library = useSoundLibrary();
  const sounds = useClipSounds(clips.map((clip) => clip.sound));
  const latestRef = useRef({ edit, steps });
  latestRef.current = { edit, steps };
  const writer = audioClipWriter(latestRef);
  const adding = useSoundAdding({
    latestRef,
    library,
    playhead,
    select,
    timelineMs,
    writer,
  });
  const chosen =
    selection?.kind === "audioClip"
      ? (clips.find((clip) => clip.id === baseOf(selection.id)) ?? null)
      : null;
  const durationOf = (clip: RecordingAudioClip) =>
    sounds.get(clip.sound)?.durationMs ??
    clip.offsetMs + clip.endMs - clip.startMs;

  const changeAudioClip = (change: AudioClipChange) => {
    if (!chosen) return;
    if (change.delete) {
      writer.update(chosen.id, () => null);
      select(null);
      return;
    }
    if (change.replace) {
      adding.replaceSound(chosen.id).catch((cause: unknown) => {
        console.error("Could not replace the sound", cause);
      });
      return;
    }
    const { isLooped, offsetMs, ...values } = change;
    writer.update(chosen.id, (clip) => {
      let next = clip;
      if (isLooped !== undefined)
        next = loopedClip(next, isLooped, durationOf(next));
      if (offsetMs !== undefined)
        next = startedFrom(next, offsetMs, durationOf(next));
      return { ...next, ...values };
    });
  };

  useEditorWindowShortcuts({
    onDelete: chosen
      ? () => {
          changeAudioClip({ delete: true });
        }
      : undefined,
  });

  const shownClip = (clip: RecordingAudioClip) => ({
    ...clip,
    endMs: Math.min(clip.endMs, timelineMs),
  });
  const audioClips: TimelineAudioClip[] = clips.flatMap((clip) =>
    recordedParts(time.map, time.runs, shownClip(clip)).map((span, part) => ({
      ...span,
      id: partId(clip.id, part),
      isLooped: clip.isLooped,
      kind: clip.kind,
      lane: clip.lane,
      title: sounds.get(clip.sound)?.title ?? "",
    })),
  );
  const strip: Pick<TimelineStripProps, "audio" | "onAudioDrop"> = {
    audio: {
      clips: audioClips,
      onRestack: (laneOf) => {
        const lanes = new Map<string, number>();
        for (const { id } of audioClips)
          if (id in laneOf && !lanes.has(baseOf(id)))
            lanes.set(baseOf(id), laneOf[id]);
        const current = latestRef.current.edit.audioClips ?? [];
        writer.write(
          current.map((clip) => ({
            ...clip,
            lane: lanes.get(clip.id) ?? clip.lane,
          })),
        );
      },
      onRetime: (id, { end, start }, part) => {
        // Pulled, a clip left past a shorter timeline is kept to it.
        writer.update(baseOf(id), (clip, others) => {
          const shown = shownClip(clip);
          if (part === "body") {
            const was = audioClips.find((each) => each.id === id);
            if (!was) return clip;
            const deltaMs =
              playedMs(time.map, start) - playedMs(time.map, was.start);
            return movedClip(others, shown, { deltaMs, timelineMs });
          }
          const pulled = retimedParts(time, shown, { end, id, start });
          return trimmedClip(shown, {
            atMs: part === "start" ? pulled.startMs : pulled.endMs,
            bounds: trimBounds(others, shown, {
              durationMs: durationOf(clip),
              edge: part,
              timelineMs,
            }),
            edge: part,
          });
        });
      },
    },
    onAudioDrop: adding.onAudioDrop,
  };
  const island: ToolPanelStripItem | null = chosen && {
    clipMs: Math.min(chosen.endMs, timelineMs) - chosen.startMs,
    id: chosen.id,
    isLooped: chosen.isLooped,
    isSong: chosen.kind === "music",
    kind: "audioClip",
    makesWay: chosen.makesWay,
    offsetMs: chosen.offsetMs,
    sound: chosen.sound,
    soundMs: durationOf(chosen),
    title: sounds.get(chosen.sound)?.title ?? "",
    volumeDb: chosen.volumeDb,
  };
  const muted = edit.mutedAudio ?? [];
  // Music first, as the Tracks island lists it.
  const soundKinds = (["music", "effect"] as const).filter((kind) =>
    clips.some((clip) => clip.kind === kind),
  );
  const hearSounds = (kind: AudioClipKind, isHeard: boolean) => {
    const { edit, steps } = latestRef.current;
    const others = (edit.mutedAudio ?? []).filter((each) => each !== kind);
    steps.once({ ...edit, mutedAudio: isHeard ? others : [...others, kind] });
  };
  return {
    addSound: adding.addSound,
    changeAudioClip,
    hearSounds,
    island,
    sounds: soundKinds.map((kind) => ({
      isHeard: !muted.includes(kind),
      kind,
    })),
    strip,
  };
}
