// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ProgressPanel } from "../../composites/progress-panel/progress-panel";
import { t } from "../../i18n/i18n";
import { Keyboard, Shortcut } from "../../primitives/keyboard/keyboard";

import { ScrollingCapturePhase } from "./scrolling-capture-events";

const CANCEL_KEY = "Escape";

const phaseLabel = (phase: ScrollingCapturePhase | undefined) => {
  if (phase === "capturing") return t("screenshots-scrolling-capturing");
  if (phase === "stitching") return t("screenshots-scrolling-stitching");
  return t("screenshots-scrolling-working");
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
  const label = finished
    ? t("screenshots-scrolling-finishing")
    : phaseLabel(phase);

  return (
    // What is happening, the bar, and the key that stops it.
    <main className="window-surface flex h-full w-full items-center overflow-hidden rounded-window p-section text-content-fg">
      <ProgressPanel
        label={label}
        progress={null}
        progressLabel={t("screenshots-scrolling-progress")}
        status={
          cancellable && !finished ? (
            <span className="flex items-center gap-inline whitespace-nowrap">
              <Shortcut>
                <Keyboard>{CANCEL_KEY}</Keyboard>
              </Shortcut>
              <span>{t("screenshots-scrolling-cancel-hint")}</span>
            </span>
          ) : undefined
        }
      />
    </main>
  );
}
