// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircularProgress } from "../../components/base/circular-progress/circular-progress";
import { Keyboard, Shortcut } from "../../components/base/keyboard/keyboard";

import { ScrollingCapturePhase } from "./scrolling-capture-events";

const phaseLabels: Record<ScrollingCapturePhase, string> = {
  capturing: "Capturing",
  stitching: "Stitching",
  working: "Working",
};

type ScrollingCaptureOverlayProps = {
  cancellable: boolean;
  finished?: boolean;
  phase?: ScrollingCapturePhase;
};

/**
 * The window shown over the region being captured. The backend cannot know how
 * far a page scrolls before it stops, so there is no percentage to show and the
 * spinner carries the whole "still working" signal.
 */
export function ScrollingCaptureOverlay({
  cancellable,
  finished = false,
  phase,
}: ScrollingCaptureOverlayProps) {
  const label = finished ? "Finishing" : phase ? phaseLabels[phase] : "Working";

  return (
    <main className="window-surface gap-section p-section flex h-full w-full items-center overflow-hidden rounded-window text-content-fg">
      <CircularProgress
        aria-label="Scrolling capture progress"
        isIndeterminate
      />
      <div className="gap-tight flex flex-col">
        <span className="text-sm">{label}</span>
        {cancellable && !finished ? (
          <span className="gap-control flex items-center whitespace-nowrap text-xs text-muted">
            <Shortcut>
              <Keyboard>Esc</Keyboard>
            </Shortcut>
            <span>to cancel</span>
          </span>
        ) : null}
      </div>
    </main>
  );
}
