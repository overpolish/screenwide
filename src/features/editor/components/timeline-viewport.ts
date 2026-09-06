// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * Viewport-only geometry for the export timeline.
 *
 * `zoom` is a unitless scale (1 means the complete timeline fits in the
 * viewport). `panOffset` is the normalized timeline fraction at the left edge
 * of the viewport. `rect.left`, `rect.width`, `x`, and pixel deltas are CSS
 * pixels; fractions and pan are unitless. The functions are immutable so the
 * result can be kept directly in React state.
 */

export const TIMELINE_MIN_ZOOM = 1;
export const TIMELINE_MAX_ZOOM = 20;

export interface TimelineViewportState {
  panOffset: number;
  zoom: number;
}

export interface TimelineViewportRect {
  left: number;
  width: number;
}

export const fitTimelineViewport = (): TimelineViewportState => ({
  panOffset: 0,
  zoom: TIMELINE_MIN_ZOOM,
});

const clamp = (value: number, min: number, max: number): number =>
  Math.max(min, Math.min(max, value));

export const clampTimelineZoom = (zoom: number): number =>
  Number.isFinite(zoom)
    ? clamp(zoom, TIMELINE_MIN_ZOOM, TIMELINE_MAX_ZOOM)
    : TIMELINE_MIN_ZOOM;

export const clampTimelinePan = (panOffset: number, zoom: number): number => {
  const z = clampTimelineZoom(zoom);
  return Number.isFinite(panOffset) ? clamp(panOffset, 0, 1 - 1 / z) : 0;
};

export const normalizeTimelineViewport = (
  state: Partial<TimelineViewportState> = {},
): TimelineViewportState => {
  const zoom = clampTimelineZoom(state.zoom ?? TIMELINE_MIN_ZOOM);
  return {
    panOffset: clampTimelinePan(state.panOffset ?? 0, zoom),
    zoom,
  };
};

/** Converts a timeline fraction to a CSS-pixel x coordinate. */
export const timelineFractionToX = (
  fraction: number,
  state: TimelineViewportState,
  rect: TimelineViewportRect,
): number => {
  const viewport = normalizeTimelineViewport(state);
  return (
    rect.left + (fraction - viewport.panOffset) * viewport.zoom * rect.width
  );
};

/** Converts a CSS-pixel x coordinate to a normalized timeline fraction. */
export const timelineXToFraction = (
  x: number,
  state: TimelineViewportState,
  rect: TimelineViewportRect,
): number => {
  const viewport = normalizeTimelineViewport(state);
  if (rect.width <= 0 || !Number.isFinite(rect.width))
    return viewport.panOffset;
  return viewport.panOffset + (x - rect.left) / (viewport.zoom * rect.width);
};

/**
 * Zooms by a multiplicative factor while keeping the timeline fraction under
 * `cursorX` fixed. The anchor is exact unless the new zoom reaches a pan edge.
 */
export const zoomTimelineViewportAt = (
  state: TimelineViewportState,
  options: {
    cursorX: number;
    factor: number;
    rect: TimelineViewportRect;
  },
): TimelineViewportState => {
  const current = normalizeTimelineViewport(state);
  const { cursorX, factor, rect } = options;
  if (
    rect.width <= 0 ||
    !Number.isFinite(rect.width) ||
    !Number.isFinite(factor) ||
    factor <= 0
  )
    return current;

  const anchor = timelineXToFraction(cursorX, current, rect);
  const zoom = clampTimelineZoom(current.zoom * factor);
  const panOffset = anchor - (cursorX - rect.left) / (zoom * rect.width);
  return { panOffset: clampTimelinePan(panOffset, zoom), zoom };
};

/**
 * Translates content by a CSS-pixel drag delta. Positive delta moves content
 * right (and therefore reveals earlier timeline content); callers handling a
 * scroll-wheel delta can pass its negation if their platform uses the inverse
 * convention.
 */
export const panTimelineViewportByPixels = (
  state: TimelineViewportState,
  deltaX: number,
  rectWidth: number,
): TimelineViewportState => {
  const current = normalizeTimelineViewport(state);
  if (!Number.isFinite(deltaX) || rectWidth <= 0 || !Number.isFinite(rectWidth))
    return current;
  return {
    panOffset: clampTimelinePan(
      current.panOffset - deltaX / (current.zoom * rectWidth),
      current.zoom,
    ),
    zoom: current.zoom,
  };
};

export const resetTimelineViewport = fitTimelineViewport;
