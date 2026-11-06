// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import {
  Check,
  Copy,
  LocateFixed,
  RotateCcw,
  SquarePen,
  X,
} from "lucide-react";
import { useNumberFormatter } from "react-aria";

import { ConfirmActionButton } from "../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../i18n/i18n";
import { IconButton } from "../../primitives/button/icon-button";
import { IconSwap } from "../../primitives/icon-swap/icon-swap";
import { Island } from "../../primitives/island/island";
import { IslandParts } from "../../primitives/island/island-parts";
import { type IslandProps } from "../../primitives/island/island-props";
import { IslandRow } from "../../primitives/island/island-row";
import { PillGroup } from "../../primitives/pill-group/pill-group";

import {
  REFERENCE_OPACITIES,
  type ReferenceOpacity,
} from "./reference-opacity";

// The island under a reference, out for as long as the reference is, so a
// reference laid exactly over what it shows reads as one: how see-through it
// is at the row's start, then putting it back where it was taken, or, with
// nowhere to put it back, back to its true size, copying it, opening it in
// the editor and closing it, each a glyph named for assistive technology
// alone, as islands show no tooltips.

export type ReferenceIslandProps = Pick<IslandProps, "isShown" | "onHidden"> & {
  /** Whether the reference was just copied, which the copy glyph says. */
  isCopied: boolean;
  onClose: () => void;
  onCopy: () => void;
  onOpacityChange: (opacity: ReferenceOpacity) => void;
  onOpenInEditor: () => void;
  /** Puts the reference back to its true size where it is. */
  onResetSize: () => void;
  /** Puts the reference back where it was taken, at its true size. */
  onToOrigin: () => void;
  opacity: ReferenceOpacity;
  /** Whether the reference was taken of an area it can be put back over;
   * one dropped, or of a whole screen or window, has only its size to reset. */
  returnsToOrigin: boolean;
};

const quiet = "text-fg-secondary enabled:hover:text-fg";

/** Lucide's square, on its 24 grid with its 2 stroke, filled as far as the
 * reference shows at `opacity`. Sized by the pill group it sits in. */
function OpacityGlyph({ opacity }: { opacity: number }) {
  return (
    <svg
      aria-hidden
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      viewBox="0 0 24 24"
    >
      <rect
        fill="currentColor"
        fillOpacity={opacity}
        height={18}
        rx={2}
        width={18}
        x={3}
        y={3}
      />
    </svg>
  );
}

export function ReferenceIsland({
  isCopied,
  isShown = true,
  onClose,
  onCopy,
  onHidden,
  onOpacityChange,
  onOpenInEditor,
  onResetSize,
  onToOrigin,
  opacity,
  returnsToOrigin,
}: ReferenceIslandProps) {
  const percent = useNumberFormatter({ style: "percent" });
  return (
    <Island
      aria-label={t("reference-island")}
      isShown={isShown}
      onHidden={onHidden}
      row={
        <IslandRow
          end={
            <IslandParts
              className="flex items-center gap-separate"
              direction="row"
            >
              {returnsToOrigin ? (
                <IconButton
                  aria-label={t("reference-to-origin")}
                  className={quiet}
                  key="origin"
                  onPress={onToOrigin}
                >
                  <LocateFixed aria-hidden className="size-icon" />
                </IconButton>
              ) : (
                <IconButton
                  aria-label={t("reference-reset-size")}
                  className={quiet}
                  key="reset-size"
                  onPress={onResetSize}
                >
                  <RotateCcw aria-hidden className="size-icon" />
                </IconButton>
              )}
              <IconButton
                aria-label={
                  isCopied ? t("reference-copied") : t("reference-copy")
                }
                className={quiet}
                key="copy"
                onPress={onCopy}
              >
                <span className="relative size-icon">
                  <IconSwap swapKey={isCopied ? "copied" : "copy"}>
                    {isCopied ? (
                      <Check aria-hidden className="size-icon" />
                    ) : (
                      <Copy aria-hidden className="size-icon" />
                    )}
                  </IconSwap>
                </span>
              </IconButton>
              <IconButton
                aria-label={t("reference-open-in-editor")}
                className={quiet}
                key="editor"
                onPress={onOpenInEditor}
              >
                <SquarePen aria-hidden className="size-icon" />
              </IconButton>
              <ConfirmActionButton
                confirmLabel={t("reference-close-confirm")}
                icon={<X aria-hidden />}
                key="close"
                label={t("reference-close")}
                onConfirm={onClose}
                variant="ghost"
              />
            </IslandParts>
          }
          start={
            <PillGroup
              aria-label={t("reference-opacity")}
              items={REFERENCE_OPACITIES.map((step) => ({
                ariaLabel: percent.format(step / 100),
                icon: <OpacityGlyph opacity={step / 100} />,
                id: String(step),
              }))}
              onSelectionChange={(step) => {
                onOpacityChange(Number(step) as ReferenceOpacity);
              }}
              selected={String(opacity)}
            />
          }
        />
      }
      shrinksToNothing
    />
  );
}
