// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  neutralScene,
  SCENE_LAYOUT_KEYS,
  sceneLayoutOf,
  withSceneLayout,
} from "./format-resolve";
import { annotationContext, isShownIn, withHere } from "./format-scope";
import {
  applied,
  changed,
  defined,
  type Formats,
  itemLayoutKey,
  same,
  tidy,
  withLayout,
  type Write,
} from "./format-write-parts";

import type { Annotation } from "../../../../bindings/Annotation";
import type { CaptionLayout } from "../../../../bindings/CaptionLayout";
import type { FormatEdits } from "../../../../bindings/FormatEdits";
import type { FormatScope } from "../../../../bindings/FormatScope";
import type { RecordingAnnotationClip } from "../../../../bindings/RecordingAnnotationClip";
import type { RecordingSceneClip } from "../../../../bindings/RecordingSceneClip";
import type { SceneFormatLayout } from "../../../../bindings/SceneFormatLayout";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

/**
 * A change made to the format in hand's timeline, `previous` to `next`,
 * written back into the one shared edit, `stored`, and what it does
 * differently per format, `edits`. Everything lands in every format:
 * what is added or deleted, timing, text, what a scene shows and how it
 * plays; leaving something out of a format is asked for apart
 * (`format-sync.ts`). A layout (where a scene's panes go, where an
 * annotation sits, how the captions look) lands where `scope` says: in
 * the shared layout every shape follows, or in the shape in hand alone.
 */
export function writtenTimelineEdit(write: Write): {
  edit: RecordingTimelineEdit;
  edits: FormatEdits;
} {
  const scenes = writtenScenes(write);
  // Annotations and captions sit on the screen, so platforms of one
  // proportion share their layouts whatever each one's framing.
  const shared = { ...write, context: annotationContext(write.context) };
  const annotations = writtenAnnotations(shared);
  const audio = writtenAudio(write);
  const captions = writtenCaptions(shared);
  const edits: FormatEdits = {
    ...write.edits,
    annotations: annotations.edits,
    audio: audio.edits,
    scenes: scenes.edits,
  };
  if (captions.edits) edits.captions = captions.edits;
  else Reflect.deleteProperty(edits, "captions");
  return {
    edit: {
      ...write.next,
      annotationClips: annotations.clips,
      audioClips: audio.clips,
      ...(captions.captions ? { captions: captions.captions } : {}),
      sceneClips: scenes.clips,
    },
    edits,
  };
}

function writtenScenes({
  context,
  edits,
  next,
  previous,
  scope,
  stored,
}: Write) {
  const all: Record<string, Formats<SceneFormatLayout> | undefined> = {
    ...edits.scenes,
  };
  const keyOf = (formats: Formats<unknown> | undefined) =>
    itemLayoutKey(formats, scope, context);
  const before = new Map(previous.sceneClips?.map((clip) => [clip.id, clip]));
  const after = new Map(next.sceneClips?.map((clip) => [clip.id, clip]));
  const clips = (stored.sceneClips ?? []).flatMap((clip) => {
    const was = before.get(clip.id);
    const now = after.get(clip.id);
    if (!was) return [clip];
    if (!now) {
      Reflect.deleteProperty(all, clip.id);
      return [];
    }
    const hidden = !isShownIn(all[clip.id]?.showIn, context);
    // Left out of this format, the scene stood as a plain full scene: a
    // change to it shows it here again, at the layout it was given.
    const shared = changed(was, now, [...SCENE_LAYOUT_KEYS]);
    let merged = applied(clip, hidden ? pickTiming(shared) : shared);
    const layoutWas = hidden
      ? sceneLayoutOf(neutralScene(was))
      : sceneLayoutOf(was);
    if (!same(layoutWas, sceneLayoutOf(now)) || (hidden && !same(was, now))) {
      if (hidden)
        all[clip.id] = {
          ...all[clip.id],
          showIn: withHere(all[clip.id]?.showIn, [context.id], context),
        };
      const key = keyOf(all[clip.id]);
      if (key === null) merged = withSceneLayout(merged, sceneLayoutOf(now));
      all[clip.id] = withLayout(all[clip.id], {
        key,
        layout: sceneLayoutOf(now),
        own: context.shape,
      });
    }
    const tidied = tidy(all[clip.id] ?? {});
    if (tidied) all[clip.id] = tidied;
    else Reflect.deleteProperty(all, clip.id);
    return [merged];
  });
  const known = new Set(clips.map((clip) => clip.id));
  for (const clip of next.sceneClips ?? [])
    if (!before.has(clip.id) && !known.has(clip.id)) clips.push(clip);
  clips.sort((left, right) => left.startMs - right.startMs);
  return { clips, edits: defined(all) };
}

/** The timing fields alone, for a scene that stood neutral here. */
const pickTiming = (fields: Partial<RecordingSceneClip>) => {
  const { endMs, startMs } = fields;
  return {
    ...(endMs === undefined ? {} : { endMs }),
    ...(startMs === undefined ? {} : { startMs }),
  };
};

function writtenAnnotations({
  context,
  edits,
  next,
  previous,
  scope,
  stored,
}: Write) {
  const all: Record<string, Formats<Annotation> | undefined> = {
    ...edits.annotations,
  };
  const keyOf = (formats: Formats<unknown> | undefined) =>
    itemLayoutKey(formats, scope, context);
  const idOf = (clip: RecordingAnnotationClip) => clip.annotation.id;
  const before = new Map(
    previous.annotationClips?.map((clip) => [idOf(clip), clip]),
  );
  const after = new Map(
    next.annotationClips?.map((clip) => [idOf(clip), clip]),
  );
  const clips = (stored.annotationClips ?? []).flatMap((clip) => {
    const id = idOf(clip);
    const was = before.get(id);
    const now = after.get(id);
    if (!was) return [clip];
    if (!now) {
      Reflect.deleteProperty(all, id);
      return [];
    }
    let merged = applied(clip, changed(was, now, ["annotation"]));
    if (!same(was.annotation, now.annotation)) {
      const key = keyOf(all[id]);
      if (key === null) merged = { ...merged, annotation: now.annotation };
      all[id] = withLayout(all[id], {
        key,
        layout: now.annotation,
        own: context.shape,
      });
    }
    const tidied = tidy(all[id] ?? {});
    if (tidied) all[id] = tidied;
    else Reflect.deleteProperty(all, id);
    return [merged];
  });
  const known = new Set(clips.map(idOf));
  for (const clip of next.annotationClips ?? [])
    if (!before.has(idOf(clip)) && !known.has(idOf(clip))) clips.push(clip);
  return { clips, edits: defined(all) };
}

/** Songs and sound effects play the same wherever they play: every change
 * to one is shared, as are those added and deleted. */
function writtenAudio({ edits, next, previous, stored }: Write) {
  const all: Record<string, { showIn?: FormatScope } | undefined> = {
    ...edits.audio,
  };
  const before = new Map(previous.audioClips?.map((clip) => [clip.id, clip]));
  const after = new Map(next.audioClips?.map((clip) => [clip.id, clip]));
  const clips = (stored.audioClips ?? []).flatMap((clip) => {
    const was = before.get(clip.id);
    const now = after.get(clip.id);
    if (!was) return [clip];
    if (now) return [applied(clip, changed(was, now, []))];
    Reflect.deleteProperty(all, clip.id);
    return [];
  });
  const known = new Set(clips.map((clip) => clip.id));
  for (const clip of next.audioClips ?? [])
    if (!before.has(clip.id) && !known.has(clip.id)) clips.push(clip);
  return { clips, edits: defined(all) };
}

function writtenCaptions({
  context,
  edits,
  next,
  previous,
  scope,
  stored,
}: Write) {
  const was = previous.captions;
  const now = next.captions;
  const own = stored.captions;
  if (!was || !now || !own)
    return { captions: now ?? own, edits: edits.captions };
  let formats = edits.captions;
  let captions = applied(
    own,
    changed(was, now, ["enabled", "layout", "style"]),
  );
  // Captions turned on or off are so in every format.
  if (was.enabled !== now.enabled) {
    captions = { ...captions, enabled: now.enabled };
    formats = tidy({ ...formats, showIn: undefined });
  }
  if (!same(was.style, now.style) || !same(was.layout, now.layout)) {
    const key = itemLayoutKey(formats, scope, context);
    const look: CaptionLayout = {
      style: now.style,
      ...(now.layout ? { layout: now.layout } : {}),
    };
    if (key === null)
      captions = { ...captions, layout: now.layout ?? {}, style: now.style };
    formats = tidy(
      withLayout(formats, { key, layout: look, own: context.shape }),
    );
  }
  return { captions, edits: formats };
}
