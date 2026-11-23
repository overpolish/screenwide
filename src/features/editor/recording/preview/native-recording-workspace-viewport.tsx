// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject } from "react";

import { t } from "../../../../i18n/i18n";
import { CircularProgress } from "../../../../primitives/circular-progress/circular-progress";

/** A pane's geometry is expressed in the unscaled workspace coordinate space. */
type NativeRecordingWorkspacePane = {
  height: number;
  index: number;
  label: string;
  ref: RefObject<HTMLCanvasElement | null>;
  width: number;
  x: number;
  y: number;
};

type NativeRecordingWorkspaceViewportProps = {
  ariaLabel: string;
  isBusy: boolean;
  panes: NativeRecordingWorkspacePane[];
  workspaceHeight: number;
  workspaceWidth: number;
  /**
   * The audio-only preview, whose surface draws the ribbon over the whole
   * viewport. It has no panes, so the layout finds this element by its marker
   * rather than through a pane canvas.
   */
  audioOnly?: boolean;
};

/** What a recording arrangement hands the viewport: its marker panes and the
 * workspace they are laid out in. */
export type NativeRecordingWorkspace = Pick<
  NativeRecordingWorkspaceViewportProps,
  "ariaLabel" | "panes" | "workspaceHeight" | "workspaceWidth"
>;

const VIEWPORT_GUTTER = 8;

/**
 * Passive native recording surface.
 *
 * This deliberately has no pointer or wheel handlers. The native compositor
 * owns the panes; the canvases here are marker elements whose layout is
 * measured by useRecordingPreviewSurface.
 */
export function NativeRecordingWorkspaceViewport({
  ariaLabel,
  audioOnly = false,
  isBusy,
  panes,
  workspaceHeight,
  workspaceWidth,
}: NativeRecordingWorkspaceViewportProps) {
  // The panes are fitted inside the gutter by CSS against the viewport's own
  // size, never through React state: the page measures them as the window
  // is resized, before a re-render could catch up, and markers a render
  // behind the viewport they are measured in told the native side a fit a
  // step out of date, which a zoomed picture showed as a jump.
  const ratio =
    workspaceWidth > 0 && workspaceHeight > 0
      ? workspaceWidth / workspaceHeight
      : 0;
  const gutters = `${(VIEWPORT_GUTTER * 2).toString()}px`;
  const percent = (part: number, whole: number) =>
    `${((part / whole) * 100).toString()}%`;

  return (
    <div
      aria-label={ariaLabel}
      className="relative flex min-h-0 grow overflow-hidden rounded-[inherit]"
      data-audio-ribbon-viewport={audioOnly ? "" : undefined}
      data-recording-preview-viewport
      dir="ltr"
      role="img"
      style={{ containerType: "size" }}
    >
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 -z-10 bg-recess"
        data-preview-backdrop
      />
      {ratio > 0 ? (
        <div
          className="absolute shrink-0 -translate-x-1/2 -translate-y-1/2"
          style={{
            aspectRatio: ratio.toString(),
            left: "50%",
            top: "50%",
            width: `max(0px, min(100cqw - ${gutters}, (100cqh - ${gutters}) * ${ratio.toString()}))`,
          }}
        >
          {panes.map((pane) => (
            <canvas
              aria-label={pane.label}
              className="pointer-events-none absolute max-w-none opacity-0"
              key={pane.index}
              ref={pane.ref}
              role="img"
              style={{
                height: percent(pane.height, workspaceHeight),
                left: percent(pane.x, workspaceWidth),
                top: percent(pane.y, workspaceHeight),
                width: percent(pane.width, workspaceWidth),
              }}
            />
          ))}
        </div>
      ) : null}
      {isBusy ? (
        <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
          <CircularProgress
            aria-label={t("editor-recording-preparing-preview")}
            isIndeterminate
          />
        </div>
      ) : null}
    </div>
  );
}
