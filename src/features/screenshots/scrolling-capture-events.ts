// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The events the scrolling capture emits while it drives the user's window.
 * The overlay window is the only listener: the capture owns the pointer for
 * several seconds, so this is the user's whole view of what is happening.
 *
 * The finished event carries no outcome. An unalignable seam is stitched
 * best-effort rather than failed, and the failures that remain are reported by
 * the capture command rejecting, so the overlay only needs to know it is over.
 */
export const SCROLLING_CAPTURE_PROGRESS_EVENT = "scrolling-capture://progress";
export const SCROLLING_CAPTURE_FINISHED_EVENT = "scrolling-capture://finished";

export type ScrollingCapturePhase = "capturing" | "stitching" | "working";

export type ScrollingCaptureProgressEvent = {
  phase: ScrollingCapturePhase;
};
