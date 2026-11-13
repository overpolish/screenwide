// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useSyncExternalStore } from "react";

// A picture dragged over the editor, which the page follows only as points
// (`use-picture-drop.ts`), and the annotation lanes it lands in: the lanes
// carry it as a bar from the moment it reaches them, the others gliding
// round it, and say where it would land (`annotation-drop-lane.tsx`). Let
// go there, the picture is placed from that moment, and once its image is
// on the strip the lanes put it in the lane it was let go in.

/** The id the carried picture's bar goes by in the lanes. */
export const INCOMING_PICTURE = "incoming-picture";

/** Where a picture let go now lands: the moment of the edited video, in ms,
 * and every bar's lane, its own under `INCOMING_PICTURE`. */
export type PictureLanding = { atMs: number; lanes: Record<string, number> };

let point: { x: number; y: number } | null = null;
let landing: PictureLanding | null = null;
let placing: Record<string, number> | null = null;
const listeners = new Set<() => void>();

const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

/** Where on the page, in CSS px, a picture is carried, or null with none. */
export function carryPictureAt(next: { x: number; y: number } | null) {
  if (next?.x === point?.x && next?.y === point?.y) return;
  point = next;
  if (!next) landing = null;
  for (const listener of listeners) listener();
}

/** The point a picture is carried at, for the lanes to follow. */
export const usePictureCarry = () =>
  useSyncExternalStore(subscribe, () => point);

/** Calls `listener` with each point a picture is carried to, or null once
 * it is gone; returns what stops it. */
export const onPictureCarry = (
  listener: (at: { x: number; y: number } | null) => void,
) =>
  subscribe(() => {
    listener(point);
  });

/** Says where the carried picture would land; null off the lanes. */
export function setPictureLanding(next: PictureLanding | null) {
  landing = next;
}

/** Where the carried picture would land if let go now. */
export const pictureLanding = () => landing;

/** Asks the lanes to put the picture being placed in `lanes` once its
 * image is on the strip, or lets that go (null). */
export function expectPlacedPicture(lanes: Record<string, number> | null) {
  placing = lanes;
}

/** The lanes the picture being placed goes in, once, taken by the lanes as
 * its image arrives. */
export function takePlacedPicture() {
  const lanes = placing;
  placing = null;
  return lanes;
}

/** Whether a picture is being placed, its lanes not yet taken. */
export const isPlacingPicture = () => placing !== null;
