// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Plays a flow through the editor as a person would do it, slowly enough to
// follow: a pointer you can see moves, hovers, presses and drags, and a line
// at the top says what is being done. It sends the pointer events a real
// pointer sends, to whatever is under it, so the editor answers exactly as
// it answers a hand: outlines, the lens, snapping and all. When the tour is
// done the editor is left as it ended, to carry on by hand.

type Target = Element | { x: number; y: number };

const ARROW = `<svg width="22" height="22" viewBox="0 0 24 24"><path d="M3 2 L3 19 L8 14 L12 22 L15 20 L11 13 L18 13 Z" fill="#111" stroke="#fff" stroke-width="1.5"/></svg>`;

const ease = (t: number) => 1 - (1 - t) ** 3;

const centreOf = (target: Target) => {
  if (!(target instanceof Element)) return target;
  const box = target.getBoundingClientRect();
  return { x: box.left + box.width / 2, y: box.top + box.height / 2 };
};

export function startTour(root: HTMLElement) {
  const view = root.ownerDocument.defaultView ?? window;
  const doc = root.ownerDocument;

  // A pointer the page did not see go down cannot be captured; the tour's
  // presses are not the browser's own, so capture is let pass quietly and
  // the tour sends the moves to the pressed element itself.
  const { prototype } = view.Element;
  const capture = Object.getOwnPropertyDescriptor(
    prototype,
    "setPointerCapture",
  )?.value as (this: Element, id: number) => void;
  prototype.setPointerCapture = function (this: Element, id: number) {
    try {
      capture.call(this, id);
    } catch {
      // Not a pointer the browser knows: the tour stands in for capture.
    }
  };

  const pointer = doc.createElement("div");
  pointer.innerHTML = ARROW;
  Object.assign(pointer.style, {
    filter: "drop-shadow(0 1px 2px rgb(0 0 0 / 0.45))",
    left: "0",
    pointerEvents: "none",
    position: "fixed",
    top: "0",
    transition: "scale 120ms ease-out",
    zIndex: "2147483647",
  });
  const caption = doc.createElement("div");
  Object.assign(caption.style, {
    background: "var(--color-island)",
    borderRadius: "999px",
    color: "white",
    font: "500 13px/18px system-ui, sans-serif",
    left: "50%",
    opacity: "0",
    padding: "7px 14px",
    pointerEvents: "none",
    position: "fixed",
    top: "58px",
    transform: "translateX(-50%)",
    transition: "opacity 180ms ease-out",
    whiteSpace: "nowrap",
    zIndex: "2147483647",
  });
  root.append(pointer, caption);

  let at = { x: view.innerWidth / 2, y: view.innerHeight / 2 };
  let over: Element | null = null;
  let pressed: Element | null = null;

  const place = () => {
    pointer.style.transform = `translate(${String(at.x - 3)}px, ${String(at.y - 2)}px)`;
  };
  place();

  const send = (type: string, target: Element, related?: Element | null) => {
    const init = {
      bubbles: true,
      button: 0,
      buttons: pressed ? 1 : 0,
      cancelable: true,
      clientX: at.x,
      clientY: at.y,
      composed: true,
      relatedTarget: related ?? null,
      view,
    };
    target.dispatchEvent(
      type.startsWith("pointer")
        ? new view.PointerEvent(type, {
            ...init,
            height: 1,
            isPrimary: true,
            pointerId: 1,
            pointerType: "mouse",
            width: 1,
          })
        : new view.MouseEvent(type, init),
    );
  };

  /** Tells what is under the pointer that it came or went, as hover does. */
  const hover = () => {
    const under = doc.elementFromPoint(at.x, at.y);
    if (under !== over) {
      if (over) {
        send("pointerout", over, under);
        send("mouseout", over, under);
      }
      if (under) {
        send("pointerover", under, over);
        send("mouseover", under, over);
      }
      over = under;
    }
    const target = pressed ?? under;
    if (target) {
      send("pointermove", target);
      send("mousemove", target);
    }
  };

  const wait = (ms: number) =>
    new Promise<void>((resolve) => {
      view.setTimeout(resolve, ms);
    });

  /** Glides to the target. It is looked up again every frame, since things
   * can move on the way (the strip's lens opening around what was just
   * chosen), so the pointer lands on where the target is, not where it
   * was. */
  const moveTo = (target: Target | (() => Target), ms = 700) => {
    const from = at;
    const began = view.performance.now();
    return new Promise<void>((resolve) => {
      const step = (now: number) => {
        const t = Math.min(1, (now - began) / ms);
        const to = centreOf(typeof target === "function" ? target() : target);
        at = {
          x: from.x + (to.x - from.x) * ease(t),
          y: from.y + (to.y - from.y) * ease(t),
        };
        place();
        hover();
        if (t < 1) view.requestAnimationFrame(step);
        else resolve();
      };
      view.requestAnimationFrame(step);
    });
  };

  const press = () => {
    const target = doc.elementFromPoint(at.x, at.y);
    if (!target) return;
    pressed = target;
    pointer.style.scale = "0.85";
    send("pointerdown", target);
    send("mousedown", target);
  };

  const release = (isClick: boolean) => {
    const target = pressed;
    if (!target) return;
    pressed = null;
    pointer.style.scale = "1";
    send("pointerup", target);
    send("mouseup", target);
    if (isClick) send("click", target);
  };

  return {
    /** Moves to it and presses it. */
    click: async (target: Target | (() => Target), ms?: number) => {
      await moveTo(target, ms);
      await wait(160);
      press();
      await wait(90);
      release(true);
      await wait(260);
    },
    /** Moves to it and presses it twice, quickly. */
    doubleClick: async (target: Target | (() => Target), ms?: number) => {
      await moveTo(target, ms);
      await wait(160);
      press();
      release(true);
      await wait(80);
      press();
      const pressedTwice = pressed;
      release(true);
      if (pressedTwice) send("dblclick", pressedTwice);
      await wait(300);
    },
    /** Presses where the pointer is, then drags to `to`. */
    drag: async (to: () => Target, ms = 1_100) => {
      press();
      await wait(180);
      await moveTo(to, ms);
      await wait(220);
      release(false);
      await wait(260);
    },
    end: async () => {
      await wait(2_400);
      caption.style.opacity = "0";
      pointer.remove();
      // The pointer goes, so what it was over hears it leave, as the
      // moment preview does, and closes.
      if (over) {
        send("pointerout", over);
        send("mouseout", over);
        over = null;
      }
      prototype.setPointerCapture = capture;
      await wait(200);
      caption.remove();
    },
    moveTo: async (target: Target | (() => Target), ms?: number) => {
      await moveTo(target, ms);
    },
    /** Says what comes next, and gives it a moment to be read. */
    say: async (text: string, ms = 1_400) => {
      caption.textContent = text;
      caption.style.opacity = "1";
      await wait(ms);
    },
    wait,
  };
}

/** Waits for something to be on the page, as the editor mounts. */
export async function find<T extends Element>(
  lookUp: () => T | null | undefined,
  ms = 4_000,
) {
  const began = Date.now();
  for (;;) {
    const found = lookUp();
    if (found) return found;
    if (Date.now() - began > ms) throw new Error("The tour found nothing");
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
}
