// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Keyboard, Shortcut } from "../../components/base/keyboard/keyboard";
import { ProgressPanel } from "../../components/shared/progress-panel/progress-panel";

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
    <main className="window-surface p-section flex h-full w-full items-center overflow-hidden rounded-window text-content-fg">
      <ProgressPanel
        label={label}
        orientation="row"
        progress={null}
        progressLabel="Scrolling capture progress"
        secondary={
          cancellable && !finished ? (
            <span className="gap-control flex items-center whitespace-nowrap">
              <Shortcut>
                <Keyboard>Esc</Keyboard>
              </Shortcut>
              <span>to cancel</span>
            </span>
          ) : undefined
        }
      />
    </main>
  );
}
