// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  OverlayScrollbarsComponent,
  type OverlayScrollbarsComponentRef,
} from "overlayscrollbars-react";
import { useCallback, useEffect, useRef, type ReactNode } from "react";

import { cn } from "../../lib/styling";

import { getEdgeFades } from "./scroll-area-edges";

import type { OverlayScrollbars } from "overlayscrollbars";

/** How far the content fades at an end with more beyond it, in px. */
const FADE_LENGTH = 16;

type ScrollAreaProps = {
  children?: ReactNode;
  className?: string;
  constrainHeight?: boolean;
  /** The edge a horizontal scroll starts from follows this direction. `auto`
   * takes it from the content's own letters, for a strip that holds a name. */
  dir?: "auto" | "ltr" | "rtl";
  edgeEffect?: "fade" | "none";
  /** Called once the viewport is set up and scrolls, which happens a moment
   * after mounting: what is to be scrolled into view first waits for it. */
  onInitialized?: (viewport: HTMLElement) => void;
  orientation?: "horizontal" | "vertical";
  rootClassName?: string;
  scrollbarAutoHide?: "scroll" | "never";
  /** Scroll without a visible scrollbar, for strips with no room for one. */
  scrollbarHidden?: boolean;
};

// Where there is more to scroll, the content itself fades out at that end, a
// mask on the viewport rather than a band painted over it, so it reads the
// same on any background: a window, a field, a native panel's material. The
// fade grows with the first stretch scrolled from an end, and a horizontal
// strip's ends turn with its writing direction.
export function ScrollArea({
  children,
  className,
  constrainHeight,
  dir,
  edgeEffect = "fade",
  onInitialized,
  orientation = "vertical",
  rootClassName,
  scrollbarAutoHide = "scroll",
  scrollbarHidden = false,
}: ScrollAreaProps) {
  const scrollRef = useRef<OverlayScrollbarsComponentRef>(null);
  const horizontal = orientation === "horizontal";

  const updateEdges = useCallback(
    (instance: OverlayScrollbars) => {
      const { viewport } = instance.elements();
      const position = horizontal ? viewport.scrollLeft : viewport.scrollTop;
      const maximum = horizontal
        ? viewport.scrollWidth - viewport.clientWidth
        : viewport.scrollHeight - viewport.clientHeight;
      const rtl = horizontal && getComputedStyle(viewport).direction === "rtl";
      const fade = getEdgeFades(rtl ? -position : position, maximum, {
        effect: edgeEffect,
        length: FADE_LENGTH,
      });
      const mask =
        fade.start === 0 && fade.end === 0
          ? ""
          : `linear-gradient(${
              horizontal ? (rtl ? "to left" : "to right") : "to bottom"
            }, transparent 0, #000 ${String(fade.start)}px, #000 calc(100% - ${String(
              fade.end,
            )}px), transparent 100%)`;
      viewport.style.setProperty("mask-image", mask);
      viewport.style.setProperty("-webkit-mask-image", mask);
    },
    [edgeEffect, horizontal],
  );

  // Also refresh after changing the effect in Storybook or a parent render.
  useEffect(() => {
    const instance = scrollRef.current?.osInstance();
    if (instance) updateEdges(instance);
  }, [updateEdges]);

  return (
    <div
      className={cn("relative h-full w-full overflow-hidden", rootClassName)}
      dir={dir}
    >
      <OverlayScrollbarsComponent
        className={cn("h-full w-full", constrainHeight && "max-h-[inherit]")}
        defer
        events={{
          initialized: (instance) => {
            updateEdges(instance);
            onInitialized?.(instance.elements().viewport);
          },
          scroll: updateEdges,
          updated: updateEdges,
        }}
        options={{
          overflow: {
            x: horizontal ? "scroll" : "hidden",
            y: horizontal ? "hidden" : "scroll",
          },
          scrollbars: {
            autoHide: scrollbarAutoHide,
            theme: "os-theme-screenwide",
            visibility: scrollbarHidden ? "hidden" : "auto",
          },
        }}
        ref={scrollRef}
      >
        <div className={cn(horizontal && "text-nowrap", className)}>
          {children}
        </div>
      </OverlayScrollbarsComponent>
    </div>
  );
}
