// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircularProgress } from "../../components/base/circular-progress/circular-progress";
import { Keyboard } from "../../components/base/keyboard/keyboard";

import { ScrollingCapturePhase } from "./scrolling-capture-events";

const phaseLabels: Record<ScrollingCapturePhase, string> = {
  capturing: "Capturing…",
  stitching: "Stitching…",
  working: "Working…",
};

type ScrollingCaptureOverlayProps = {
  cancellable: boolean;
  finished?: boolean;
  phase?: ScrollingCapturePhase;
};

/**
 * The card shown over the region being captured. The backend cannot know how
 * far a page scrolls before it stops, so there is no percentage to show and the
 * spinner carries the whole "still working" signal.
 */
export function ScrollingCaptureOverlay({
  cancellable,
  finished = false,
  phase,
}: ScrollingCaptureOverlayProps) {
  const label = finished
    ? "Finishing…"
    : phase
      ? phaseLabels[phase]
      : "Working…";

  return (
    <main className="flex h-full w-full flex-col items-center justify-center gap-3 overflow-hidden">
      <CircularProgress
        aria-label="Scrolling capture progress"
        isIndeterminate
        size="large"
      />
      {/*
        Only the text is backed. The ring reads clearly against whatever it is
        over, so darkening the whole region would hide more of the page than it
        helps.
      */}
      <div className="flex flex-col items-center gap-0.5 rounded-md bg-content/92 px-3 py-2">
        <span className="text-sm text-content-fg">{label}</span>
        {cancellable && !finished ? (
          <span className="flex items-center gap-1 text-xs text-muted">
            <Keyboard size="sm">Esc</Keyboard>
            to cancel
          </span>
        ) : null}
      </div>
    </main>
  );
}
