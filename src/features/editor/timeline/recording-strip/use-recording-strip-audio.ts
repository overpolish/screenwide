// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { useEditorWindowShortcuts } from "../../shortcuts/use-editor-window-shortcuts";
import {
  type AudioClipKind,
  keptStretch,
  loopedClip,
  movedClip,
  type RecordingAudioClip,
  retypedClip,
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
    const { isLooped, kind, lengthMs, offsetMs, ...values } = change;
    writer.update(chosen.id, (clip, others) => {
      let next = clip;
      if (kind !== undefined)
        next = retypedClip(others, next, { kind, timelineMs });
      if (isLooped !== undefined)
        next = loopedClip(next, isLooped, durationOf(next));
      if (offsetMs !== undefined)
        next =
          lengthMs === undefined
            ? startedFrom(next, offsetMs, durationOf(next))
            : keptStretch(
                next,
                { lengthMs, offsetMs },
                { durationMs: durationOf(next), timelineMs },
              );
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
  /** `clip` as the strip draws it, a part each side of every jump. */
  const partsOf = (clip: RecordingAudioClip, title: string) =>
    recordedParts(time.map, time.runs, shownClip(clip)).map((span, part) => ({
      ...span,
      id: partId(clip.id, part),
      isLooped: clip.isLooped,
      kind: clip.kind,
      lane: clip.lane,
      title,
    }));
  const audioClips: TimelineAudioClip[] = clips.flatMap((clip) =>
    partsOf(clip, sounds.get(clip.sound)?.title ?? ""),
  );
  const { carried } = adding;
  const strip: Pick<TimelineStripProps, "audio"> = {
    audio: {
      carried: carried && {
        ghosts: carried.ghosts.flatMap((ghost) => partsOf(ghost, ghost.title)),
        kind: carried.kind,
      },
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
          { isDragged: true },
        );
      },
      onRetime: (id, { end, start }, part) => {
        // A drag tells every move, and they are one step of history
        // together. Pulled, a clip left past a shorter timeline is kept to
        // it; a sound effect moved whole keeps its length, its tail past the
        // end unheard.
        writer.update(
          baseOf(id),
          (clip, others) => {
            const shown = shownClip(clip);
            if (part === "body") {
              const was = audioClips.find((each) => each.id === id);
              if (!was) return clip;
              const deltaMs =
                playedMs(time.map, start) - playedMs(time.map, was.start);
              return movedClip(others, clip.kind === "music" ? shown : clip, {
                deltaMs,
                timelineMs,
              });
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
          },
          { isDragged: true },
        );
      },
    },
  };
  const island: ToolPanelStripItem | null = chosen && {
    clipMs: Math.min(chosen.endMs, timelineMs) - chosen.startMs,
    hasSongRoom:
      retypedClip(clips, chosen, { kind: "music", timelineMs }).kind ===
      "music",
    id: chosen.id,
    isInstrumental: chosen.isInstrumental,
    isLooped: chosen.isLooped,
    isSong: chosen.kind === "music",
    kind: "audioClip",
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
    hasSongRoom: adding.hasSongRoom,
    hearSounds,
    island,
    sounds: soundKinds.map((kind) => ({
      isHeard: !muted.includes(kind),
      kind,
    })),
    strip,
  };
}
