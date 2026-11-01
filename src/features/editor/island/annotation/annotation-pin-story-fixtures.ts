// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AnnotationPinAction, AnnotationPinView } from "./annotation-pin";

// A pin's states at the playhead, for the annotation islands' stories, and
// what each of the island's asks turns one into, so a story's toggle and
// answers play out as the editor's would.

export const PIN_FOLLOWING: AnnotationPinView = {
  action: "hide",
  hasCorrections: false,
  progress: null,
  state: "following",
  unsure: null,
};

export const PIN_OUT_OF_VIEW: AnnotationPinView = {
  action: "show",
  hasCorrections: false,
  progress: null,
  state: "out-of-view",
  unsure: null,
};

export const PIN_WITH_CORRECTIONS: AnnotationPinView = {
  ...PIN_FOLLOWING,
  hasCorrections: true,
};

/** Where the tracker lost the content, after it was put right once. */
export const PIN_UNSURE: AnnotationPinView = {
  ...PIN_WITH_CORRECTIONS,
  unsure: "lost",
};

export const PIN_TRACKING: AnnotationPinView = {
  action: "hide",
  hasCorrections: false,
  progress: 0.4,
  state: "tracking",
  unsure: null,
};

/** `pin` once the island has asked for `action`. */
export const storyPinAfter = (
  pin: AnnotationPinView | null | undefined,
  action: AnnotationPinAction,
): AnnotationPinView | null => {
  if (action === "unpin" || !pin)
    return action === "pin" ? PIN_FOLLOWING : null;
  if (action === "clear") return { ...pin, hasCorrections: false };
  if (action === "hide")
    return { ...pin, action: "show", state: "out-of-view" };
  if (action === "show")
    return { ...pin, action: "hide", hasCorrections: true, state: "following" };
  return pin;
};
