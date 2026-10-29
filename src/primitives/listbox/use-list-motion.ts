// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import { useEffect } from "react";

import { arrive, glide, reveal } from "../island/island-motion";

// Items coming and going in a list: those that stay glide to their new
// places on the island's curve, and one arriving is revealed in its place
// once the others have mostly made room, as an island's parts are
// (`IslandFlow`). React Aria renders the items and takes one out of its
// collection the moment it goes, so they cannot be motion components kept
// while they leave; instead the list's DOM is watched, and each change is
// played from where the items were (first, last, invert, play). The DOM
// rather than a render: React Aria may lay its items out in a render of its
// own after the list's. A mutation's callback runs before the frame is
// painted, so nothing is drawn in its new place first.
//
// The items are the list's children, and those of its sections. A
// virtualized list wraps each item in a box of its own that it places
// itself, so its items are not among them and are left alone.

const ITEMS = ":scope > :not([role=group]), :scope > [role=group] > *";

const ms = (seconds: number) => seconds * 1000;

/** Where the element is laid out down the list, leaving out any glide it is
 * part way through. */
const layoutTop = (element: HTMLElement, listTop: number, shift: number) =>
  element.getBoundingClientRect().top - listTop - shift;

export function useListMotion(list: HTMLElement | null) {
  const reduce = useReducedMotion() ?? false;
  useEffect(() => {
    if (!list || reduce) return;
    const tops = new WeakMap<Element, number>();
    const glides = new WeakMap<Element, Animation>();
    const shiftOf = (item: HTMLElement) => {
      const running = glides.get(item);
      if (running?.playState !== "running") return 0;
      return new DOMMatrixReadOnly(getComputedStyle(item).transform).m42;
    };
    // Where everything is now, without moving anything: at the start, and
    // when the list changes size for a reason of its own, such as a face
    // loading, so that change is never played as a move later.
    const measure = () => {
      const listTop = list.getBoundingClientRect().top;
      for (const item of list.querySelectorAll<HTMLElement>(ITEMS))
        tops.set(item, layoutTop(item, listTop, shiftOf(item)));
    };
    const play = () => {
      const listTop = list.getBoundingClientRect().top;
      const moves = [...list.querySelectorAll<HTMLElement>(ITEMS)].map(
        (item) => {
          const shift = shiftOf(item);
          return {
            from: tops.get(item),
            item,
            shift,
            top: layoutTop(item, listTop, shift),
          };
        },
      );
      for (const { from, item, shift, top } of moves) {
        tops.set(item, top);
        if (from === undefined) {
          item.animate([reveal.initial, reveal.animate], {
            delay: ms(arrive.delay),
            duration: ms(arrive.duration),
            easing: "ease-out",
            fill: "backwards",
          });
          continue;
        }
        const offset = from + shift - top;
        if (Math.abs(offset) < 0.5) continue;
        glides.get(item)?.cancel();
        glides.set(
          item,
          item.animate(
            [
              { transform: `translateY(${String(offset)}px)` },
              { transform: "none" },
            ],
            { duration: ms(glide.duration), easing: "ease-out" },
          ),
        );
      }
    };
    measure();
    const mutations = new MutationObserver(play);
    mutations.observe(list, { childList: true, subtree: true });
    const resizes = new ResizeObserver(measure);
    resizes.observe(list);
    return () => {
      mutations.disconnect();
      resizes.disconnect();
    };
  }, [list, reduce]);
}
