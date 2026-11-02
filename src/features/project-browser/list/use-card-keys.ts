// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type RefObject, useEffect, useRef } from "react";

import { isWindows } from "../../../lib/platform";
import { editsText } from "../../editor/shortcuts/keyboard-target";

// The keyboard among the cards, as Finder's icon view has it: the arrows
// move between pictures, along a row and up and down the columns; Command
// (Control on Windows) with O or the down arrow opens the one focused, as
// Return does through the picture's own press.

const PICTURE = "[data-project-picture]";

const fileOf = (element: Element | null) =>
  element?.closest<HTMLElement>("[data-project-file]")?.dataset.projectFile;

/** The picture to go to from `from` by `key`, among `pictures` in the order
 * they are laid out. */
function nextPicture(
  pictures: HTMLElement[],
  from: HTMLElement,
  { isRtl, key }: { isRtl: boolean; key: string },
) {
  const index = pictures.indexOf(from);
  if (key === "ArrowLeft" || key === "ArrowRight") {
    const ahead = (key === "ArrowRight") !== isRtl;
    return pictures[index + (ahead ? 1 : -1)] as HTMLElement | undefined;
  }
  const box = from.getBoundingClientRect();
  const middle = box.left + box.width / 2;
  const isDown = key === "ArrowDown";
  const rows = pictures
    .map((picture) => ({ box: picture.getBoundingClientRect(), picture }))
    .filter(({ box: other }) =>
      isDown ? other.top > box.bottom - 1 : other.bottom < box.top + 1,
    );
  if (rows.length === 0) return undefined;
  // The nearest row that way, then the picture in it nearest the column.
  const rowTop = isDown
    ? Math.min(...rows.map(({ box: other }) => other.top))
    : Math.max(...rows.map(({ box: other }) => other.top));
  return rows
    .filter(({ box: other }) => Math.abs(other.top - rowTop) < 2)
    .sort(
      (one, other) =>
        Math.abs(one.box.left + one.box.width / 2 - middle) -
        Math.abs(other.box.left + other.box.width / 2 - middle),
    )[0]?.picture;
}

export function useCardKeys({
  onOpen,
  rootRef,
}: {
  /** Absent where projects cannot be opened, as in Recently Deleted. */
  onOpen: ((file: string) => void) | undefined;
  rootRef: RefObject<HTMLElement | null>;
}) {
  const keysRef = useRef<(event: KeyboardEvent) => void>(() => undefined);
  useEffect(() => {
    keysRef.current = (event) => {
      const root = rootRef.current;
      if (!root || event.defaultPrevented || editsText(event.target)) return;
      const focused =
        document.activeElement instanceof HTMLElement &&
        document.activeElement.matches(PICTURE) &&
        root.contains(document.activeElement)
          ? document.activeElement
          : null;
      const command = isWindows() ? event.ctrlKey : event.metaKey;
      if (
        focused &&
        command &&
        (event.key === "ArrowDown" || event.key.toLowerCase() === "o")
      ) {
        const file = fileOf(focused);
        if (!file || !onOpen) return;
        event.preventDefault();
        onOpen(file);
      } else if (focused && !command && event.key.startsWith("Arrow")) {
        const pictures = [...root.querySelectorAll<HTMLElement>(PICTURE)];
        const next = nextPicture(pictures, focused, {
          isRtl: getComputedStyle(root).direction === "rtl",
          key: event.key,
        });
        event.preventDefault();
        next?.focus();
      }
    };
  });
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      keysRef.current(event);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);
}
