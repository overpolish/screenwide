// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type RefObject, useLayoutEffect } from "react";

/** Where `query` first falls in `text`, as `[from, to)` letters, without
 * regard to case; null where it does not, or nothing is asked for. */
export const findIn = (
  text: string,
  query: string,
): [number, number] | null => {
  const needle = query.toLocaleLowerCase();
  if (needle === "") return null;
  const index = text.toLocaleLowerCase().indexOf(needle);
  return index === -1 ? null : [index, index + needle.length];
};

// One highlight holds every field's found letters; `::highlight(found-text)`
// in `utilities.css` paints it as the `found-text` utility paints a piece.
const HIGHLIGHT = "found-text";
let found: Highlight | null = null;
const foundHighlight = () => {
  if (typeof CSS === "undefined" || !("highlights" in CSS)) return null;
  if (!found) {
    found = new Highlight();
    CSS.highlights.set(HIGHLIGHT, found);
  }
  return found;
};

/** Paints where `query` falls in the text of `ref`'s element as found, as
 * `found-text` does, without touching the element's content: for a field
 * that is edited in place. Follows the text as it is typed in. */
export function useFoundHighlight(
  ref: RefObject<HTMLElement | null>,
  query: string | undefined,
) {
  useLayoutEffect(() => {
    const element = ref.current;
    const highlight = foundHighlight();
    if (!element || !highlight || !query) return;
    // Static, not live: a live range collapses when something moves the
    // field within the page, as a scroll area does as it sets up. A change
    // to the text itself is followed by marking it again.
    let range: StaticRange | null = null;
    const mark = () => {
      if (range) highlight.delete(range);
      range = null;
      const node = element.firstChild;
      if (!(node instanceof Text)) return;
      const match = findIn(node.data, query);
      // Letters whose lower case is longer can carry a match past the end.
      if (!match || match[1] > node.length) return;
      range = new StaticRange({
        endContainer: node,
        endOffset: match[1],
        startContainer: node,
        startOffset: match[0],
      });
      highlight.add(range);
    };
    mark();
    const observer = new MutationObserver(mark);
    observer.observe(element, {
      characterData: true,
      childList: true,
      subtree: true,
    });
    return () => {
      observer.disconnect();
      if (range) highlight.delete(range);
    };
  }, [query, ref]);
}
