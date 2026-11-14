// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// No scene plays over a gap: a gap is a blank scene of its own, which shows
// its background and what is drawn over it, so a scene of the recording
// stops at a gap's edges as it does at another scene's. One that would
// cross a gap, drawn before this held or brought over it some other way, is
// cut back to either side of it.

type Span = { endMs: number; startMs: number };

/** Shorter than this, in ms, a scene would spend its whole life arriving. */
export const LEAST_SCENE_MS = 500;

const crosses = (clip: Span, gap: Span) =>
  gap.startMs < clip.endMs && gap.endMs > clip.startMs;

/**
 * `clips` cut back to either side of the `gaps` they cross, in timeline
 * order; a side shorter than a scene can be is dropped. The first side kept
 * keeps the clip's id and each later one takes its own, which `onPart` hears
 * with the clip it was cut from. `clips` itself where none crosses a gap.
 */
export function scenesClearOfGaps<Clip extends Span & { id: string }>(
  clips: Clip[],
  gaps: readonly Span[],
  onPart?: (from: Clip, part: Clip) => void,
): Clip[] {
  if (!clips.some((clip) => gaps.some((gap) => crosses(clip, gap))))
    return clips;
  return clips.flatMap((clip) => {
    const sides: Span[] = [];
    let start = clip.startMs;
    for (const gap of gaps.filter((gap) => crosses(clip, gap))) {
      if (gap.startMs > start)
        sides.push({ endMs: gap.startMs, startMs: start });
      start = Math.max(start, gap.endMs);
    }
    if (start < clip.endMs) sides.push({ endMs: clip.endMs, startMs: start });
    return sides
      .filter((side) => side.endMs - side.startMs >= LEAST_SCENE_MS)
      .map((side, index) => {
        const part = {
          ...clip,
          ...side,
          id: index === 0 ? clip.id : crypto.randomUUID(),
        };
        if (index > 0) onPart?.(clip, part);
        return part;
      });
  });
}
