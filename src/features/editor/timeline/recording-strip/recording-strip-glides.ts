// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { annotationGlides } from "../../recording/annotations/recording-annotation-glides";
import { type TimelineAnnotation } from "../annotation-lane/annotation-lane";
import { baseOf } from "../strip/strip-part";
import { type Span } from "../timeline-scale";

import type { RecordingSceneClip } from "../../../../bindings/RecordingSceneClip";
import type { RecordingAnnotationClip } from "../../recording/annotations/recording-annotations";

type Size = { height: number; width: number };

/**
 * `bars` with where each one's annotation travels between its places in
 * the scenes (`glides`), in recorded seconds, kept to each bar's part.
 * `partsOf` lays timeline milliseconds out on the strip, as the bars' own
 * parts are, either side of a jump; `pictures` are the screen's and the
 * camera's source sizes, which a glide is paced against, and `videoMs` the
 * edited video's length.
 */
export function withAnnotationGlides(
  bars: TimelineAnnotation[],
  {
    clips,
    partsOf,
    pictures,
    scenes,
    videoMs,
  }: {
    clips: readonly RecordingAnnotationClip[];
    partsOf: (span: { endMs: number; startMs: number }) => Span[];
    pictures: { camera?: Size; primary?: Size };
    scenes: readonly RecordingSceneClip[];
    videoMs: number | undefined;
  },
): TimelineAnnotation[] {
  if (!clips.some((clip) => clip.sceneShapes?.length)) return bars;
  const sorted = [...scenes].sort((a, b) => a.startMs - b.startMs);
  const glides = new Map(
    clips.map((clip) => [
      clip.annotation.id,
      annotationGlides({
        clip,
        picture: clip.trackId === "camera" ? pictures.camera : pictures.primary,
        scenes: sorted,
        videoMs,
      }).flatMap((glide) => partsOf(glide)),
    ]),
  );
  return bars.map((bar) => {
    const kept = (glides.get(baseOf(bar.id)) ?? []).flatMap((stretch) => {
      const start = Math.max(stretch.start, bar.start);
      const end = Math.min(stretch.end, bar.end);
      return end > start ? [{ end, start }] : [];
    });
    return kept.length > 0 ? { ...bar, glides: kept } : bar;
  });
}
