// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { keyLabel } from "../../components/base/keyboard/key-label";
import { Keyboard, Shortcut } from "../../components/base/keyboard/keyboard";
import { ProgressPanel } from "../../components/shared/progress-panel/progress-panel";
import { t } from "../../i18n/i18n";

import { ScrollingCapturePhase } from "./scrolling-capture-events";

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
    // The Finder copy-dialog column: phrase, bar, and the way out beneath.
    <main className="window-surface flex h-full w-full items-center overflow-hidden rounded-window p-section text-content-fg">
      <ProgressPanel
        label={label}
        progress={null}
        progressLabel={t("screenshots-scrolling-progress")}
        secondary={
          cancellable && !finished ? (
            <span className="flex items-center gap-control whitespace-nowrap">
              <Shortcut>
                <Keyboard>{keyLabel("escape")}</Keyboard>
              </Shortcut>
              <span>{t("screenshots-scrolling-cancel-hint")}</span>
            </span>
          ) : undefined
        }
      />
    </main>
  );
}
