// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PreviewOutputSize, PreviewZoomField } from "./preview-readouts";

/**
 * The screenshot workspace's status bar: what the picture measures and how
 * close it is drawn, along the bottom of the window.
 *
 * The still has no transport, so this is the recording timeline band's first
 * row standing on its own - same fill, same inset, same 24px rhythm - so the
 * two workspaces close on the same line.
 */
export function ScreenshotStatusBar({
  height,
  onZoomChange,
  width,
  zoomPercent,
}: {
  height: number;
  onZoomChange: (zoomPercent: number) => void;
  width: number;
  zoomPercent: number;
}) {
  return (
    <div className="shrink-0 bg-fill-quaternary px-window-inset">
      <div className="flex min-h-control-height items-center gap-section py-control-inset">
        <PreviewOutputSize height={height} width={width} />
        <PreviewZoomField onChange={onZoomChange} zoomPercent={zoomPercent} />
      </div>
    </div>
  );
}
