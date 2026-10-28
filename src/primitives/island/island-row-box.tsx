// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { type ReactNode } from "react";
import { Toolbar } from "react-aria-components";

import { ISLAND } from "./island-geometry";
import { focusIn } from "./island-motion";

// The row's own box, coming into focus with the island and leaving with it.
// Never narrower than its content, so the width read for it is what it
// needs, whatever the layer it sits in is at that moment. Where it fills the
// island it spans the row's layer, which spans the black as it glides; the
// span is left out when its width is read (`useSize`'s `isNatural`).

export function IslandRowBox({
  children,
  fillsWidth,
  isShown,
  label,
  onElement,
}: {
  children: ReactNode;
  fillsWidth: boolean;
  isShown: boolean;
  label: string;
  onElement: (element: HTMLDivElement | null) => void;
}) {
  return (
    <AnimatePresence initial={false}>
      {isShown ? (
        <motion.div
          key="row"
          {...focusIn}
          className="flex w-max shrink-0"
          initial={false}
          ref={onElement}
          style={{
            padding: ISLAND.rowInset,
            ...(fillsWidth ? { minWidth: "100%" } : {}),
          }}
        >
          <Toolbar
            aria-label={label}
            className={fillsWidth ? "flex grow" : "flex"}
          >
            {children}
          </Toolbar>
        </motion.div>
      ) : null}
    </AnimatePresence>
  );
}
