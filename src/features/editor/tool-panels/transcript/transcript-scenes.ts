// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  sceneCustomLabel,
  scenePresetLabel,
} from "../../recording/scenes/recording-scene-labels";
import {
  SceneChoice,
  sceneSpanTarget,
  SceneSpanTarget,
} from "../../recording/scenes/recording-scene-span";
import { SceneOffer } from "../scene/scene-offer";

import { TranscriptUnit, WordSelection } from "./transcript-units";

import type {
  TranscriptPlayed,
  TranscriptRange,
  TranscriptScene,
} from "../tool-panel-transcript";

const TEMPLATE_PREFIX = "template:";

/** What the words' menu offers for scenes: whether a pick adds one or
 * changes the one the words sit in, and each layout by its name. */
export type TranscriptSceneMenu = {
  options: { id: string; label: string }[];
  type: SceneSpanTarget["type"];
};

/**
 * Where `ranges` of the recording play on the edited video `played` lays
 * out, as one stretch from the first moment any plays to the last, with the
 * video's length. What a cut left out plays nowhere; the span is null where
 * nothing of `ranges` plays.
 */
export function transcriptTimelineSpan(
  played: readonly TranscriptPlayed[],
  ranges: readonly TranscriptRange[],
): { lengthMs: number; span: TranscriptRange | null } {
  let atMs = 0;
  let span: TranscriptRange | null = null;
  for (const stretch of played) {
    const rate = stretch.rate ?? 1;
    for (const range of ranges) {
      const start = Math.max(range.startMs, stretch.startMs);
      const end = Math.min(range.endMs, stretch.endMs);
      if (end <= start) continue;
      const startMs = atMs + (start - stretch.startMs) / rate;
      const endMs = atMs + (end - stretch.startMs) / rate;
      span = span
        ? {
            endMs: Math.max(span.endMs, endMs),
            startMs: Math.min(span.startMs, startMs),
          }
        : { endMs, startMs };
    }
    atMs += (stretch.endMs - stretch.startMs) / rate;
  }
  return { lengthMs: atMs, span };
}

/**
 * What a scene chosen for `span` of the shown `units` acts on, or null where
 * it can act on nothing. Struck words are left out. A scene that places the
 * camera stands paused while the camera is not drawn into the video, and is
 * not changed from here, as the Scene panel offers nothing over one.
 */
export function transcriptSceneTarget({
  offer,
  played,
  scenes,
  span,
  units,
}: {
  offer: SceneOffer;
  played: readonly TranscriptPlayed[];
  scenes: readonly TranscriptScene[];
  span: WordSelection;
  units: readonly TranscriptUnit[];
}): SceneSpanTarget | null {
  const kept = units
    .slice(span.first, span.last + 1)
    .filter((unit) => !unit.removed);
  const timed = transcriptTimelineSpan(played, kept);
  if (!timed.span) return null;
  const target = sceneSpanTarget({
    clips: scenes,
    lengthMs: timed.lengthMs,
    span: timed.span,
  });
  if (target?.type !== "change" || offer.canPlaceCamera) return target;
  const paused = scenes.some(
    (scene) => scene.id === target.id && scene.needsCamera,
  );
  return paused ? null : target;
}

/** The menu's scene group for `target`, in the Scene panel's order: Custom,
 * the presets, then your templates. */
export const transcriptSceneMenu = (
  offer: SceneOffer,
  target: SceneSpanTarget,
): TranscriptSceneMenu => ({
  options: [
    { id: "custom", label: sceneCustomLabel() },
    ...offer.presets.map((preset) => ({
      id: preset,
      label: scenePresetLabel(preset),
    })),
    ...offer.templates.map((template) => ({
      id: `${TEMPLATE_PREFIX}${template.id}`,
      label: template.name,
    })),
  ],
  type: target.type,
});

/** The layout a menu option `id` names, or null where it is no longer on
 * offer. */
export function transcriptSceneChoice(
  offer: SceneOffer,
  id: string,
): SceneChoice | null {
  if (id === "custom") return { type: "custom" };
  if (id.startsWith(TEMPLATE_PREFIX)) {
    const templateId = id.slice(TEMPLATE_PREFIX.length);
    const template = offer.templates.find(({ id }) => id === templateId);
    return template ? { template, type: "template" } : null;
  }
  const preset = offer.presets.find((preset) => preset === id);
  return preset ? { preset, type: "preset" } : null;
}
