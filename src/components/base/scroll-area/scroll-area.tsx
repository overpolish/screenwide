// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  OverlayScrollbarsComponent,
  type OverlayScrollbarsComponentRef,
} from "overlayscrollbars-react";
import { useCallback, useEffect, useRef, type ReactNode } from "react";

import { cn } from "../../../lib/styling";

import { getEdgeOpacities } from "./scroll-area-edges";

import type { OverlayScrollbars } from "overlayscrollbars";

type ScrollAreaProps = {
  children?: ReactNode;
  className?: string;
  constrainHeight?: boolean;
  edgeEffect?: "shadow" | "inset" | "none";
  orientation?: "horizontal" | "vertical";
  rootClassName?: string;
  scrollbarAutoHide?: "scroll" | "never";
};

export function ScrollArea({
  children,
  className,
  constrainHeight,
  edgeEffect = "shadow",
  orientation = "vertical",
  rootClassName,
  scrollbarAutoHide = "scroll",
}: ScrollAreaProps) {
  const scrollRef = useRef<OverlayScrollbarsComponentRef>(null);
  const startRef = useRef<HTMLDivElement>(null);
  const endRef = useRef<HTMLDivElement>(null);
  const horizontal = orientation === "horizontal";

  const updateEdges = useCallback(
    (instance: OverlayScrollbars) => {
      const { viewport } = instance.elements();
      if (!startRef.current || !endRef.current) return;
      const position = horizontal ? viewport.scrollLeft : viewport.scrollTop;
      const maximum = horizontal
        ? viewport.scrollWidth - viewport.clientWidth
        : viewport.scrollHeight - viewport.clientHeight;
      const rtl = horizontal && getComputedStyle(viewport).direction === "rtl";
      const opacity = getEdgeOpacities(rtl ? -position : position, maximum, {
        effect: edgeEffect === "inset" ? "shadow" : edgeEffect,
      });
      // Start/end overlays are positioned at physical left/right edges.
      startRef.current.style.setProperty(
        "opacity",
        String(rtl ? opacity.end : opacity.start),
      );
      endRef.current.style.setProperty(
        "opacity",
        String(rtl ? opacity.start : opacity.end),
      );
    },
    [edgeEffect, horizontal],
  );

  // Also refresh after changing the effect in Storybook or a parent render.
  useEffect(() => {
    const instance = scrollRef.current?.osInstance();
    if (instance) updateEdges(instance);
  }, [updateEdges]);

  const edge = (side: "start" | "end") => {
    const start = side === "start";
    return (
      <div
        aria-hidden
        className={cn(
          "pointer-events-none absolute z-100 rounded-[inherit]",
          !horizontal && start && "rounded-tl-md",
          horizontal ? "inset-y-0" : "inset-x-0",
          horizontal
            ? start
              ? "left-0"
              : "right-0"
            : start
              ? "top-0"
              : "bottom-0",
          "from-shadow to-transparent opacity-0",
          horizontal ? "w-[10px]" : "h-[10px]",
          horizontal
            ? start
              ? "bg-gradient-to-r"
              : "bg-gradient-to-l"
            : start
              ? "bg-gradient-to-b"
              : "bg-gradient-to-t",
        )}
        ref={start ? startRef : endRef}
      />
    );
  };

  return (
    <div
      className={cn(
        "relative h-full w-full overflow-hidden",
        edgeEffect === "inset" && "rounded-window",
        rootClassName,
      )}
    >
      <OverlayScrollbarsComponent
        className={cn("h-full w-full", constrainHeight && "max-h-[inherit]")}
        defer
        events={{
          initialized: updateEdges,
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
            visibility: "auto",
          },
        }}
        ref={scrollRef}
      >
        <div
          className={cn(
            horizontal && "text-nowrap",
            edgeEffect === "inset" && "p-section",
            className,
          )}
        >
          {children}
        </div>
      </OverlayScrollbarsComponent>
      {edgeEffect === "inset" && (
        <div
          aria-hidden
          className="inset-shadow-full pointer-events-none absolute inset-0 z-100 rounded-[inherit]"
        />
      )}
      {edgeEffect !== "none" && edge("start")}
      {edgeEffect !== "none" && edge("end")}
    </div>
  );
}
