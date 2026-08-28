// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { OverlayScrollbarsComponent } from "overlayscrollbars-react";
import { useRef } from "react";
import { VariantProps } from "tailwind-variants";

import { cn } from "../../../lib/styling";
import { tv } from "../../../lib/variants";

import type { OverlayScrollbars } from "overlayscrollbars";

const overflowShadowVariants = tv({
  compoundSlots: [
    {
      class:
        "pointer-events-none absolute z-100 rounded-[inherit] from-shadow to-transparent",
      slots: ["end", "start"],
    },
    {
      class: "w-full h-[10px]",
      orientation: "vertical",
      slots: ["end", "start"],
    },
    {
      class: "w-[10px] inset-y-0",
      orientation: "horizontal",
      slots: ["end", "start"],
    },
  ],
  defaultVariants: {
    orientation: "vertical",
  },
  slots: {
    end: "",
    os: "relative h-full w-full overflow-hidden",
    start: "",
  },
  variants: {
    orientation: {
      horizontal: {
        end: "right-0 bg-gradient-to-l",
        start: "left-0 bg-gradient-to-r",
      },
      vertical: {
        end: "bottom-0 bg-gradient-to-t",
        start: "top-0 bg-gradient-to-b",
      },
    },
  },
});

type OverflowShadowProps = VariantProps<typeof overflowShadowVariants> & {
  children?: React.ReactNode;
  className?: string;
  constrainHeight?: boolean;
  rootClassName?: string;
};

export const OverflowShadow = ({
  children,
  className,
  constrainHeight,
  orientation,
  rootClassName,
}: OverflowShadowProps) => {
  const { end, os, start } = overflowShadowVariants({ orientation });

  const startRef = useRef<HTMLDivElement>(null);
  const endRef = useRef<HTMLDivElement>(null);

  const updateShadows = (instance: OverlayScrollbars) => {
    const { viewport } = instance.elements();
    if (!startRef.current || !endRef.current) return;

    const scrollPosition =
      orientation === "horizontal" ? viewport.scrollLeft : viewport.scrollTop;
    const scrollSize =
      orientation === "horizontal"
        ? viewport.scrollWidth
        : viewport.scrollHeight;
    const clientSize =
      orientation === "horizontal"
        ? viewport.clientWidth
        : viewport.clientHeight;
    const maxScroll = scrollSize - clientSize;

    if (maxScroll > 0) {
      const scrollAmount = scrollPosition / maxScroll;
      startRef.current.style.opacity = scrollAmount.toString();
      endRef.current.style.opacity = (1 - scrollAmount).toString();
    } else {
      startRef.current.style.opacity = "0";
      endRef.current.style.opacity = "0";
    }
  };

  return (
    <div className={cn(os(), rootClassName)}>
      <OverlayScrollbarsComponent
        className={cn("h-full w-full", constrainHeight && "max-h-[inherit]")}
        defer
        events={{
          initialized: updateShadows,
          scroll: updateShadows,
          updated: updateShadows,
        }}
        options={{
          overflow: {
            x: orientation === "horizontal" ? "scroll" : "hidden",
            y: orientation === "horizontal" ? "hidden" : "scroll",
          },
          scrollbars: {
            autoHide: "scroll",
            theme: "os-theme-screenwide",
            visibility: "visible",
          },
        }}
      >
        <div
          className={cn(
            "p-focus-safe",
            orientation === "horizontal" && "text-nowrap",
            className,
          )}
        >
          {children}
        </div>
      </OverlayScrollbarsComponent>

      <div className={start()} ref={startRef} />
      <div className={end()} ref={endRef} />
    </div>
  );
};
