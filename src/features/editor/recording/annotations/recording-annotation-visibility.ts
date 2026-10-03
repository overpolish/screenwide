// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { Annotation } from "../../annotations/annotations";
import { RecordingVideoTrackId } from "../../types";

import { RecordingAnnotationClip } from "./recording-annotations";

/** The clips on the video tracks that are turned on: what the annotation
 * lane shows and what can be chosen. A track turned off keeps its
 * annotations, out of sight until it is turned on again. */
export const shownAnnotationClips = (
  clips: RecordingAnnotationClip[],
  shownTracks: ReadonlySet<RecordingVideoTrackId>,
) =>
  clips.every((clip) => shownTracks.has(clip.trackId))
    ? clips
    : clips.filter((clip) => shownTracks.has(clip.trackId));

/**
 * `clips` with an edit made to the shown ones, `next`, laid back in. Each
 * hidden clip keeps its place in the drawing order; the shown clips take the
 * places shown clips held, in their new order. A shown clip missing from
 * `next` was deleted, and one new to it goes on top.
 */
export const withHiddenAnnotationClips = (
  clips: RecordingAnnotationClip[],
  next: RecordingAnnotationClip[],
  shownTracks: ReadonlySet<RecordingVideoTrackId>,
) => {
  if (clips.every((clip) => shownTracks.has(clip.trackId))) return next;
  const kept = new Set(next.map((clip) => clip.annotation.id));
  const pending = [...next];
  const laid = clips.flatMap((clip) => {
    if (!shownTracks.has(clip.trackId)) return [clip];
    if (!kept.has(clip.annotation.id)) return [];
    const placed = pending.shift();
    return placed ? [placed] : [];
  });
  return [...laid, ...pending];
};

/** `clips` with the shown ones' annotations as `annotations` has them, by
 * id: a shown clip whose annotation is gone was deleted, and a hidden one
 * stays as it is. */
export const withShownAnnotations = (
  clips: RecordingAnnotationClip[],
  annotations: Annotation[],
  shownTracks: ReadonlySet<RecordingVideoTrackId>,
) => {
  const byId = new Map(
    annotations.map((annotation) => [annotation.id, annotation]),
  );
  return clips.flatMap((clip) => {
    if (!shownTracks.has(clip.trackId)) return [clip];
    const annotation = byId.get(clip.annotation.id);
    return annotation ? [{ ...clip, annotation }] : [];
  });
};

/** The lane's view of `clips`: the shown ones, and its edits to them, and
 * the drafts it previews while dragging, laid back among the hidden ones. */
export const laneAnnotationClips = (
  clips: RecordingAnnotationClip[],
  shownTracks: ReadonlySet<RecordingVideoTrackId>,
  {
    commit,
    preview,
  }: {
    commit: (clips: RecordingAnnotationClip[]) => void;
    preview: (clips: RecordingAnnotationClip[] | null) => void;
  },
) => ({
  clips: shownAnnotationClips(clips, shownTracks),
  onClipsChange: (next: RecordingAnnotationClip[]) => {
    commit(withHiddenAnnotationClips(clips, next, shownTracks));
  },
  onPreviewClips: (next: RecordingAnnotationClip[] | null) => {
    preview(next && withHiddenAnnotationClips(clips, next, shownTracks));
  },
});

/**
 * Lets go of the annotations on a track as it is turned off: the choice
 * keeps what it holds of the shown ones, rather than holding the hidden ones
 * out of sight and choosing them again when the track comes back.
 * `selection` chooses among the shown annotations only.
 */
export const useHiddenAnnotationsLetGo = (
  shownTracks: ReadonlySet<RecordingVideoTrackId>,
  selection: {
    onSelectedIdsChange: (ids: readonly string[]) => void;
    selectedIds: ReadonlySet<string>;
  },
) => {
  const shownKey = [...shownTracks].sort().join(":");
  const shownKeyRef = useRef(shownKey);
  const selectionRef = useRef(selection);
  selectionRef.current = selection;
  useEffect(() => {
    if (shownKeyRef.current === shownKey) return;
    shownKeyRef.current = shownKey;
    const { onSelectedIdsChange, selectedIds } = selectionRef.current;
    onSelectedIdsChange([...selectedIds]);
  }, [shownKey]);
};
