// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLayoutEffect, useState } from "react";

type Size = { height: number; width: number };

/** An element's size, read where it is laid out. `isNatural` reads its
 * width as its content would have it, without the width it is held to, as
 * a section held to the island's box is: the island sizes the box from what
 * the section needs, so it must not see the width it gave it. The limits
 * are cleared and put back within the read, before paint. Its height is
 * read as it is shown, held: content that keeps its shape, such as a tile's
 * picture, is taller held wider, and a box sized to the narrower reading
 * would cut it short. A box scrolled inside it keeps its place: laid out
 * narrower for the read, its content can be shorter, and some engines pull
 * its scroll back to fit at once. */
const readSize = (element: HTMLElement, isNatural: boolean): Size => {
  const height = element.offsetHeight;
  if (!isNatural) return { height, width: element.offsetWidth };
  const scrolled = [...element.querySelectorAll("*")].flatMap((box) =>
    box.scrollTop !== 0 || box.scrollLeft !== 0
      ? [{ box, left: box.scrollLeft, top: box.scrollTop }]
      : [],
  );
  const { maxWidth, minWidth } = element.style;
  element.style.minWidth = "0px";
  element.style.maxWidth = "none";
  const width = element.offsetWidth;
  element.style.minWidth = minWidth;
  element.style.maxWidth = maxWidth;
  for (const { box, left, top } of scrolled) {
    box.scrollTop = top;
    box.scrollLeft = left;
  }
  return { height, width };
};

/** An element's laid-out size, kept up to date as it changes. A new element
 * has no size until it is measured, so nothing is placed by what was there
 * before it; once the element is gone, its last size stays, so what was
 * there can fold away from it. Read after every render before paint, so a
 * size that changes with what it holds is known in the frame it is drawn, as
 * the island's shift is; an observer alone answers a frame late. The
 * observer catches the rest. */
export function useSize(
  element: HTMLElement | null,
  { isNatural = false }: { isNatural?: boolean } = {},
) {
  const [measured, setMeasured] = useState<{
    element: HTMLElement;
    size: Size;
  } | null>(null);
  // Every render, deliberately: the size is read where it changes. It cannot
  // loop, as an unchanged size keeps the same state and React stops there.
  // eslint-disable-next-line @eslint-react/exhaustive-deps
  useLayoutEffect(() => {
    if (!element) return;
    const { height, width } = readSize(element, isNatural);
    // eslint-disable-next-line @eslint-react/set-state-in-effect
    setMeasured((current) =>
      current?.element === element &&
      current.size.height === height &&
      current.size.width === width
        ? current
        : { element, size: { height, width } },
    );
  });
  useLayoutEffect(() => {
    if (!element) return;
    // Reading clears and restores the element's minimum width, a change
    // of its own the content watcher below is told of; those records are
    // dropped after each read, so it never answers itself.
    let mutations: MutationObserver | null = null;
    const measure = () => {
      const size = readSize(element, isNatural);
      mutations?.takeRecords();
      // A stretched element resizes with the box as it glides, while what
      // it needs stays the same: nothing to tell then.
      setMeasured((current) =>
        current?.element === element &&
        current.size.height === size.height &&
        current.size.width === size.width
          ? current
          : { element, size },
      );
    };
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    // Stretched to a minimum, the element keeps its size when what it holds
    // narrows, so the observer is not told; the content is watched instead:
    // what it holds, its text, and the classes and styles that size it, but
    // not the states a pointer passing over sets, which change nothing's
    // size and would read it at every move.
    if (isNatural) {
      mutations = new MutationObserver(measure);
      mutations.observe(element, {
        attributeFilter: ["class", "style"],
        characterData: true,
        childList: true,
        subtree: true,
      });
    }
    return () => {
      observer.disconnect();
      mutations?.disconnect();
    };
  }, [element, isNatural]);
  if (!element) return measured?.size ?? null;
  return measured?.element === element ? measured.size : null;
}
