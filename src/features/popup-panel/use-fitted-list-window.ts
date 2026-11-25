// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  currentMonitor,
  getCurrentWindow,
  LogicalPosition,
  LogicalSize,
} from "@tauri-apps/api/window";
import { RefObject, useLayoutEffect } from "react";

import {
  emptyPopupPanelHeight,
  popupPanelMaxHeight,
  popupPanelMaxWidth,
} from "./layout";

/** Marks a row's label, which truncates rather than pushing the row wider,
 * so the window can tell how much wider its row would be in full. */
export const POPUP_LABEL_ATTRIBUTE = "data-popup-label";

/**
 * How much wider the most cut-short row needs to be, in CSS px; zero where
 * every row fits. Each row is laid out once at its natural width and read
 * with fractions kept: a label truncates for a fraction of a pixel too, which
 * the whole-pixel `scrollWidth` and `clientWidth` round away.
 */
const rowShortfall = (listbox: HTMLElement) =>
  Math.max(
    0,
    ...Array.from(
      listbox.querySelectorAll<HTMLElement>(`[${POPUP_LABEL_ATTRIBUTE}]`),
      (label) => {
        const row = label.parentElement;
        if (!row) return 0;
        const shown = row.getBoundingClientRect().width;
        const width = row.style.width;
        row.style.width = "max-content";
        const natural = row.getBoundingClientRect().width;
        row.style.width = width;
        return natural - shown;
      },
    ),
  );

/**
 * Sizes the list's own window to its items: as tall as they are, up to the
 * list ceiling unless the list asks to be shown whole, and wider than it was
 * opened where a row would otherwise be cut short, up to the width ceiling.
 * The opener only knows how wide it asked for, not how wide the words, the
 * keys beside them, or a translation turn out. A window that grows keeps its
 * start edge, and stays on its monitor; one hung above its trigger keeps its
 * bottom edge too. `measure`, where given, says how tall the list wants to
 * be, for a list that lays itself out in parts of its own.
 */
export function useFittedListWindow(
  listboxRef: RefObject<HTMLElement | null>,
  {
    content,
    itemCount,
    measure,
    opensAbove = false,
    showsAllItems,
  }: {
    content: unknown;
    itemCount: number;
    showsAllItems: boolean;
    measure?: () => number;
    opensAbove?: boolean;
  },
) {
  useLayoutEffect(() => {
    if (!listboxRef.current) return;

    const window = getCurrentWindow();
    const listbox = listboxRef.current;
    const scrollContent = measure ? null : listbox.parentElement;
    let cancelled = false;

    const resize = async () => {
      const scaleFactor = await window.scaleFactor();
      const currentSize = (await window.innerSize()).toLogical(scaleFactor);
      if (cancelled) return;

      const contentHeight = scrollContent?.scrollHeight ?? listbox.scrollHeight;
      const height =
        itemCount === 0
          ? emptyPopupPanelHeight
          : measure
            ? measure()
            : showsAllItems
              ? contentHeight
              : Math.min(contentHeight, popupPanelMaxHeight);
      const shortfall = rowShortfall(listbox);
      const width =
        shortfall > 0
          ? Math.max(
              currentSize.width,
              Math.min(
                Math.ceil(currentSize.width + shortfall),
                popupPanelMaxWidth,
              ),
            )
          : currentSize.width;
      const keepsBottom = opensAbove && height !== currentSize.height;
      if (width > currentSize.width || keepsBottom) {
        const grows = width - currentSize.width;
        const position = (await window.outerPosition()).toLogical(scaleFactor);
        let x =
          document.documentElement.dir === "rtl" && grows > 0
            ? position.x - grows
            : position.x;
        const monitor = await currentMonitor();
        if (monitor) {
          const area = monitor.workArea;
          const start = area.position.toLogical(monitor.scaleFactor).x;
          const extent = area.size.toLogical(monitor.scaleFactor).width;
          x = Math.max(start, Math.min(x, start + extent - width));
        }
        const y = keepsBottom
          ? position.y + currentSize.height - height
          : position.y;
        await window.setPosition(new LogicalPosition(x, y));
      }
      await window.setSize(new LogicalSize(width, height));
    };

    void resize();

    const observer = new ResizeObserver(() => {
      void resize();
    });
    observer.observe(scrollContent ?? listbox);
    // A list in parts grows inside a box that keeps its size; each part is
    // watched for its own rows.
    if (measure)
      for (const part of listbox.querySelectorAll('[role="listbox"]'))
        observer.observe(part);

    return () => {
      cancelled = true;
      observer.disconnect();
    };
  }, [content, itemCount, listboxRef, measure, opensAbove, showsAllItems]);
}
