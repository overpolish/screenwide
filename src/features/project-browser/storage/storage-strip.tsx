// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import { useRef } from "react";
import { Button as AriaButton } from "react-aria-components";

import { t } from "../../../i18n/i18n";
import { motionDurations, motionEasings } from "../../../lib/motion";
import { cn, focusStyles, groupFocusVisible } from "../../../lib/styling";
import { Text } from "../../../primitives/text/text";
import { Toolbar } from "../../../primitives/toolbar/toolbar";
import { formatBytes } from "../../editor/duration";

import type { ProjectSummary } from "../types";

// The disk the listed projects take, as a picture of it: one segment per
// project, as wide as what it takes, largest first. Pointing at a segment,
// or focusing it, names it and lights up its card, so "what is taking all
// this space" is answered by looking; pressing it goes to the card. The
// segments are a toolbar of buttons: one stop for Tab, the arrows between
// them. Above it, how many projects there are and what they take together,
// apart by layout rather than a dot. It sits a button's inset in, so its
// text starts where the sidebar's rows and buttons start theirs.

export function StorageStrip({
  linked,
  onGoTo,
  onLink,
  projects,
}: {
  /** The project pointed at, here or at its card, by manifest. */
  linked: string | null;
  /** Goes to the project's card, as its segment is pressed. */
  onGoTo: (file: string) => void;
  onLink: (file: string | null) => void;
  projects: readonly ProjectSummary[];
}) {
  const reducedMotion = useReducedMotion() ?? false;
  // A segment that loses focus to its card while the pointer is still on
  // the strip stays lit, as pointing at it does.
  const isPointedRef = useRef(false);
  const sized = projects
    .filter(({ sizeBytes }) => (sizeBytes ?? 0) > 0)
    .sort((one, other) => (other.sizeBytes ?? 0) - (one.sizeBytes ?? 0));
  const total = sized.reduce((sum, { sizeBytes }) => sum + (sizeBytes ?? 0), 0);
  const pointed = sized.find(({ file }) => file === linked);
  if (projects.length === 0) return null;
  return (
    <div className="flex flex-col gap-inline px-inset">
      <div className="flex min-w-0 items-baseline justify-between gap-inline">
        <Text
          as="span"
          className="truncate"
          dir={pointed ? "auto" : undefined}
          variant="caption"
        >
          {pointed
            ? pointed.title
            : t("project-browser-project-count", { count: projects.length })}
        </Text>
        {total > 0 ? (
          <Text as="span" className="shrink-0 tabular-nums" variant="caption">
            {formatBytes(pointed?.sizeBytes ?? total)}
          </Text>
        ) : null}
      </div>
      {total > 0 ? (
        // Each segment is pressed across the strip's gap after it and a
        // little above and below what is drawn, so the smallest is still
        // 4 px by 16 px to hit; the overhang is taken back from the layout.
        <Toolbar
          aria-label={t("project-browser-storage")}
          className="-my-1 gap-0"
          onPointerLeave={() => {
            isPointedRef.current = false;
            onLink(null);
          }}
        >
          {sized.map(({ file, sizeBytes, title }, index) => (
            <motion.div
              className="flex min-w-1"
              key={file}
              layout={!reducedMotion}
              // Measured only as the projects change, not each time one is
              // pointed at, which draws the strip again.
              layoutDependency={total}
              style={{ flexBasis: 0, flexGrow: sizeBytes ?? 0 }}
              transition={{
                duration: motionDurations.travel,
                ease: motionEasings.out,
              }}
            >
              <AriaButton
                aria-label={t("project-browser-go-to-project", {
                  size: formatBytes(sizeBytes ?? 0),
                  title,
                })}
                className={cn(
                  "group relative flex grow cursor-default py-1 outline-none data-[focus-visible]:z-10",
                  index < sized.length - 1 && "pe-px",
                )}
                onBlur={() => {
                  if (!isPointedRef.current) onLink(null);
                }}
                onFocus={() => {
                  onLink(file);
                }}
                onHoverStart={() => {
                  isPointedRef.current = true;
                  onLink(file);
                }}
                onPress={() => {
                  onGoTo(file);
                }}
              >
                <span
                  className={cn(
                    "h-2 grow",
                    focusStyles,
                    groupFocusVisible,
                    index === 0 && "rounded-s-full",
                    index === sized.length - 1 && "rounded-e-full",
                    file === linked ? "bg-primary" : "bg-fg-tertiary",
                  )}
                />
              </AriaButton>
            </motion.div>
          ))}
        </Toolbar>
      ) : null}
    </div>
  );
}
