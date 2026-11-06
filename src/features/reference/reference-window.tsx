// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";
import { animate, motion, useMotionValue } from "motion/react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";

import { motionDurations, motionEasings } from "../../lib/motion";
import { cn } from "../../lib/styling";

import { ReferenceIsland } from "./reference-island";
import { type ReferenceOpacity } from "./reference-opacity";
import { useReferenceLayout } from "./use-reference-layout";

// A reference's window (`src-tauri/src/references.rs`). The picture is
// drawn natively, where it is moved and resized (`references/macos.rs`);
// the page draws the island under it, always out, so a reference laid
// exactly over what it shows reads as one. The window is fitted round the
// picture, the island and the room for their shadows
// (`use-reference-layout.ts`), and takes clicks on the picture and the
// island alone. The arrow keys nudge it a point, ten with Shift, to line it
// up by hand. Only the island's close, asking twice, closes it, since its
// picture goes with it: the island tucks away and the picture fades, then
// the window goes.

/** How long the copy glyph says it was copied, in ms. */
const COPIED_MS = 1_500;

const NUDGES: Partial<Record<string, [number, number]>> = {
  ArrowDown: [0, 1],
  ArrowLeft: [-1, 0],
  ArrowRight: [1, 0],
  ArrowUp: [0, -1],
};

/** Moves the window `dx` and `dy` points, at the scale of the display it is
 * on. */
async function nudge(dx: number, dy: number) {
  const window = getCurrentWindow();
  const [at, scale] = await Promise.all([
    window.outerPosition(),
    window.scaleFactor(),
  ]);
  await window.setPosition(
    new PhysicalPosition(
      at.x + Math.round(dx * scale),
      at.y + Math.round(dy * scale),
    ),
  );
}

const report = (what: string) => (error: unknown) => {
  console.error(what, error);
};

export function ReferenceWindow() {
  const [opacity, setOpacity] = useState<ReferenceOpacity>(100);
  const [isCopied, setIsCopied] = useState(false);
  const copiedRef = useRef<number | undefined>(undefined);
  const [isShown, setIsShown] = useState(true);
  /** The island's box as it began to tuck away, which it keeps so the
   * island shrinks into its own middle. It changes no size the window is
   * fitted round meanwhile, so the window holds still as it goes. */
  const [tucking, setTucking] = useState<null | {
    height: number;
    width: number;
  }>(null);
  const { flippedFrom, islandRef, layout } = useReferenceLayout();
  /** How far the island stands from its place as it glides there. */
  const flipOffset = useMotionValue(0);
  /** Whether there is somewhere to put the reference back; null until
   * asked, while the window stays hidden. */
  const [returnsToOrigin, setReturnsToOrigin] = useState<boolean | null>(null);

  useEffect(() => {
    invoke<boolean>("reference_returns_to_origin")
      .then(setReturnsToOrigin)
      .catch(report("Could not ask where the reference was taken"));
  }, []);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const step = NUDGES[event.key];
      if (!step) return;
      event.preventDefault();
      const by = event.shiftKey ? 10 : 1;
      nudge(step[0] * by, step[1] * by).catch(report("Could not move"));
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.clearTimeout(copiedRef.current);
    };
  }, []);

  // The island takes its new place at once, as the picture moves and
  // resizes, except when it goes to the other side of the picture: then it
  // starts from where it is drawn and glides there. Set before the frame is
  // drawn, so it never shows at its new place first. A glide is left to
  // finish as the island's place changes on its side, and a new one takes
  // over from it.
  useLayoutEffect(() => {
    if (flippedFrom === null || layout === null) return;
    flipOffset.jump(flippedFrom + flipOffset.get() - layout.islandY);
    animate(flipOffset, 0, {
      duration: motionDurations.reshape,
      ease: motionEasings.out,
    });
  }, [flipOffset, flippedFrom, layout]);

  return (
    <div
      className={cn(
        "relative size-full",
        (layout === null || returnsToOrigin === null) && "invisible",
      )}
      dir="ltr"
    >
      <motion.div
        className={cn(
          "absolute w-fit",
          tucking && "flex items-center justify-center",
        )}
        ref={islandRef}
        style={{
          height: tucking?.height,
          insetInlineStart: layout?.islandX ?? 0,
          top: layout?.islandY ?? 0,
          width: tucking?.width,
          y: flipOffset,
        }}
      >
        <ReferenceIsland
          isCopied={isCopied}
          isShown={isShown}
          onClose={() => {
            const island = islandRef.current;
            if (island) {
              setTucking({
                height: island.offsetHeight,
                width: island.offsetWidth,
              });
            }
            setIsShown(false);
            invoke("fade_reference").catch(report("Could not close"));
          }}
          onCopy={() => {
            invoke("copy_reference")
              .then(() => {
                setIsCopied(true);
                window.clearTimeout(copiedRef.current);
                copiedRef.current = window.setTimeout(() => {
                  setIsCopied(false);
                }, COPIED_MS);
              })
              .catch(report("Could not copy the reference"));
          }}
          onHidden={() => {
            invoke("close_reference").catch(report("Could not close"));
          }}
          onOpacityChange={(next) => {
            setOpacity(next);
            invoke("set_reference_opacity", { opacity: next / 100 }).catch(
              report("Could not change the reference's opacity"),
            );
          }}
          onOpenInEditor={() => {
            invoke("open_reference_in_editor").catch(
              report("Could not open the reference in the editor"),
            );
          }}
          onResetSize={() => {
            invoke("reset_reference_size").catch(
              report("Could not reset the reference's size"),
            );
          }}
          onToOrigin={() => {
            invoke("reference_to_origin").catch(
              report("Could not move the reference"),
            );
          }}
          opacity={opacity}
          returnsToOrigin={returnsToOrigin ?? true}
        />
      </motion.div>
    </div>
  );
}
