// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, Copy } from "lucide-react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { ReactNode, useEffect, useRef, useState } from "react";

import { t } from "../../i18n/i18n";
import { cn } from "../../lib/styling";
import { IconButton } from "../../primitives/button/icon-button";

/** How long the check stands in for the glyph after a copy. */
const CONFIRMED_MS = 2000;

type CopyButtonProps = {
  "aria-label": string;
  /** Copies; a promise keeps the button busy until it settles. */
  onCopy: () => unknown;
  /** The glyph at rest. */
  children?: ReactNode;
  isDisabled?: boolean;
};

// A copy confirms itself, as a native copy action does: the glyph pulses
// while a slow copy runs, then gives way to a check for a moment, and the
// check is said aloud as well as shown. Presses in between are ignored, so a
// copy cannot be started twice. A copy that fails goes back to rest.
export function CopyButton({
  "aria-label": label,
  children = <Copy />,
  isDisabled,
  onCopy,
}: CopyButtonProps) {
  const [status, setStatus] = useState<"copied" | "copying" | "idle">("idle");
  const reducedMotion = useReducedMotion();
  // Each press is numbered, so a copy that settles after the button has
  // moved on, or gone, leaves it alone.
  const pressRef = useRef(0);
  const isMountedRef = useRef(false);

  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  const copy = () => {
    if (status !== "idle") return;
    const press = ++pressRef.current;
    const isCurrent = () => isMountedRef.current && pressRef.current === press;
    const confirm = () => {
      if (!isCurrent()) return;
      setStatus("copied");
      setTimeout(() => {
        if (isCurrent()) setStatus("idle");
      }, CONFIRMED_MS);
    };
    const result = onCopy();
    if (result instanceof Promise) {
      setStatus("copying");
      result.then(confirm, () => {
        if (isCurrent()) setStatus("idle");
      });
      return;
    }
    confirm();
  };

  const isCopied = status === "copied";
  const fade = {
    animate: { opacity: 1, scale: 1 },
    exit: { opacity: 0, scale: 0.8 },
    initial: { opacity: 0, scale: 0.8 },
    transition: { duration: reducedMotion ? 0 : 0.15 },
  };

  return (
    <>
      <IconButton
        aria-label={label}
        isDisabled={isDisabled}
        onPress={copy}
        variant="ghost"
      >
        <span
          className={cn(
            "grid place-items-center [&>*]:col-start-1 [&>*]:row-start-1",
            status === "copying" && "animate-pulse",
          )}
        >
          <AnimatePresence initial={false}>
            <motion.span
              className="flex"
              key={isCopied ? "copied" : "rest"}
              {...fade}
            >
              {isCopied ? <Check /> : children}
            </motion.span>
          </AnimatePresence>
        </span>
      </IconButton>
      <span aria-live="polite" className="sr-only">
        {isCopied ? t("controls-copied") : ""}
      </span>
    </>
  );
}

export type { CopyButtonProps };
