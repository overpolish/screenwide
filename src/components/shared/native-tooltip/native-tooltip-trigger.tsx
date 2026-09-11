// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { ReactNode, RefObject, use, useEffect, useRef } from "react";
import {
  TooltipTrigger,
  TooltipTriggerStateContext,
} from "react-aria-components";

import { Tooltip } from "../../base/tooltip/tooltip";

import { hideTooltip, NativeTooltipContent, showTooltip } from "./api";
import { NativeTooltipLabel } from "./native-tooltip-surface";

/** The standard tooltip delay, as the in-page tooltips use. */
const DELAY_MS = 400;

type NativeTooltipTriggerProps = {
  children: ReactNode;
  tooltip: NativeTooltipContent | string;
  /** How long the pointer must rest on the control first. */
  delay?: number;
};

const asContent = (tooltip: NativeTooltipContent | string) =>
  typeof tooltip === "string" ? { label: tooltip } : tooltip;

/**
 * Puts the tooltip window on screen for as long as React Aria says the
 * tooltip is open, and takes the anchor from the control itself.
 *
 * It draws nothing: the window does. The open state, the delay, the warm-up
 * that opens the next tooltip at once, the focus-visible rule and the close on
 * press all come from the trigger this sits inside.
 */
function NativeTooltipPresenter({
  anchorRef,
  content,
}: {
  anchorRef: RefObject<HTMLSpanElement | null>;
  content: NativeTooltipContent;
}) {
  const state = use(TooltipTriggerStateContext);
  const isOpen = state?.isOpen ?? false;
  const { label, shortcut } = content;
  // Held in a ref rather than watched: the trigger state is a fresh object on
  // every render, and a tooltip must not be asked for again just because the
  // control it describes re-rendered.
  const stateRef = useRef(state);
  stateRef.current = state;

  useEffect(() => {
    const anchor = anchorRef.current;
    if (!isOpen || !anchor) return;

    const rect = anchor.getBoundingClientRect();
    showTooltip(
      { label, shortcut },
      {
        height: rect.height,
        width: rect.width,
        x: rect.x,
        y: rect.y,
      },
    ).catch((cause: unknown) => {
      console.error("Could not show the tooltip", cause);
    });

    // A tooltip describes what the pointer is over. Anything that moves the
    // control out from under it, or takes the window the control is in out of
    // play, ends the tooltip at once rather than leaving it stranded.
    const close = () => {
      stateRef.current?.close(true);
    };
    window.addEventListener("blur", close);
    window.addEventListener("scroll", close, true);
    return () => {
      window.removeEventListener("blur", close);
      window.removeEventListener("scroll", close, true);
      hideTooltip();
    };
  }, [anchorRef, isOpen, label, shortcut]);

  return null;
}

/**
 * A tooltip drawn in a window of its own.
 *
 * The Editor's preview is a native surface layered over the webview, so a
 * tooltip drawn in the page is covered by it. Outside Tauri - in Storybook -
 * there is no window to open and no surface to clear, so the in-page tooltip
 * is used instead and the stories keep working.
 */
export function NativeTooltipTrigger({
  children,
  delay = DELAY_MS,
  tooltip,
}: NativeTooltipTriggerProps) {
  const anchorRef = useRef<HTMLSpanElement>(null);
  const content = asContent(tooltip);

  return (
    <TooltipTrigger delay={delay}>
      {/* The anchor the window is placed against: it wraps the control
          tightly, so its bounds are the control's bounds. */}
      <span className="inline-flex" ref={anchorRef}>
        {children}
      </span>
      {isTauri() ? (
        <NativeTooltipPresenter anchorRef={anchorRef} content={content} />
      ) : (
        <Tooltip placement="top">
          <NativeTooltipLabel content={content} />
        </Tooltip>
      )}
    </TooltipTrigger>
  );
}
