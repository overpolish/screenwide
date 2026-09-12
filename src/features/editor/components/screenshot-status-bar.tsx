// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PreviewZoomField } from "./preview-readouts";

/**
 * The screenshot workspace's status bar: how close the picture is drawn, along
 * the bottom of the window.
 *
 * The still has no transport, so this is the recording timeline band's first
 * row standing on its own - same fill, same inset, same control rhythm - so the
 * two workspaces close on the same line.
 */
export function ScreenshotStatusBar({
  onZoomChange,
  zoomPercent,
}: {
  onZoomChange: (zoomPercent: number) => void;
  zoomPercent: number;
}) {
  return (
    <div className="flex min-h-control-height shrink-0 items-center gap-section bg-fill-quaternary px-window-inset py-control-inset">
      <PreviewZoomField onChange={onZoomChange} zoomPercent={zoomPercent} />
    </div>
  );
}
