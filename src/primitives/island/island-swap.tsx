// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import {
  Component,
  type CSSProperties,
  type ReactNode,
  type RefObject,
  useCallback,
  useRef,
  useState,
} from "react";

import { arrive, focusIn, leave, reveal } from "./island-motion";

// Leaves quickly, as a section's part does, so the two never read as laid
// over each other.
const leaving = { ...focusIn.exit, transition: leave };

type Held = {
  element: HTMLElement;
  left: number;
  parent: HTMLElement;
  top: number;
};

/** Keeps the one leaving where it was. Lifted out, it is placed where it was
 * inside its parent, but the parent itself can move as the one arriving lays
 * the row out anew: the end of a row moves over when its start's title
 * changes width. That move is read across the change, before React makes it
 * and after, which only a class's snapshot can do, and undone on the one
 * leaving (`translate`, which its own motion's `transform` leaves alone). */
class KeepsPlace extends Component<{
  children: ReactNode;
  presentRef: RefObject<HTMLElement | null>;
  swapKey: string;
}> {
  componentDidUpdate(_: unknown, __: unknown, held: Held | null) {
    if (!held) return;
    const x = held.left - held.parent.offsetLeft;
    const y = held.top - held.parent.offsetTop;
    if (x !== 0 || y !== 0) {
      held.element.style.translate = `${String(x)}px ${String(y)}px`;
    }
  }

  getSnapshotBeforeUpdate(previous: { swapKey: string }): Held | null {
    const element = this.props.presentRef.current;
    const parent = element?.offsetParent;
    if (previous.swapKey === this.props.swapKey || !element) return null;
    if (!(parent instanceof HTMLElement)) return null;
    return { element, left: parent.offsetLeft, parent, top: parent.offsetTop };
  }

  render() {
    return this.props.children;
  }
}

// Swaps what a row or the line under it holds when `swapKey` changes: the one
// leaving blurs away where it was, lifted out (`popLayout`) so only the new
// one sizes the island, and the new one is revealed once it has gone
// (`arrive`). What it first holds is revealed a beat after the black opens
// (`reveal`). An island's own parts give way to each other with
// `IslandParts`.
export function IslandSwap({
  children,
  className,
  initial = true,
  ref,
  style,
  swapKey,
}: {
  children: ReactNode;
  swapKey: string;
  className?: string;
  // Off, what it first holds is laid out in place and only revealed as the
  // black opens round it.
  /** Reveals what it first holds as it mounts. */
  initial?: boolean;
  ref?: (element: HTMLDivElement | null) => void;
  style?: CSSProperties;
}) {
  // Set while rendering rather than in an effect, so the part that arrives
  // with the first change already waits for the one leaving.
  const [seenKey, setSeenKey] = useState(swapKey);
  const [hasSwapped, setHasSwapped] = useState(false);
  if (swapKey !== seenKey) {
    setSeenKey(swapKey);
    setHasSwapped(true);
  }
  // The one shown: the one leaving lets go only as it unmounts, after the
  // one arriving has taken it.
  const presentRef = useRef<HTMLDivElement | null>(null);
  const attach = useCallback(
    (element: HTMLDivElement | null) => {
      if (element) presentRef.current = element;
      ref?.(element);
    },
    [ref],
  );
  return (
    <KeepsPlace presentRef={presentRef} swapKey={swapKey}>
      <AnimatePresence initial={initial} mode="popLayout">
        <motion.div
          animate={reveal.animate}
          className={className}
          exit={leaving}
          initial={reveal.initial}
          key={swapKey}
          ref={attach}
          style={style}
          transition={
            hasSwapped || swapKey !== seenKey ? arrive : reveal.transition
          }
        >
          {children}
        </motion.div>
      </AnimatePresence>
    </KeepsPlace>
  );
}
