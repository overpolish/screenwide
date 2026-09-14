// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { ReactNode, Ref } from "react";
import {
  Button as AriaButton,
  ToggleButton as AriaToggleButton,
  TooltipTrigger,
} from "react-aria-components";

import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";
import { Tooltip } from "../tooltip/tooltip";

type ToggleMenuButtonProps = {
  /** The glyph shown while the input is on. */
  children: ReactNode;
  /** The name of what the pop-up chooses. The trailing part is this label,
   * and pressing it opens the choices. */
  label: string;
  /** Names the trailing part, which opens the choices. */
  menuLabel: string;
  /** The glyph shown while the input is off. */
  off: ReactNode;
  /** `fromKeyboard` says the menu was opened without a pointer, so what
   * opens can take focus rather than leaving it on the button. */
  onMenuPress: (anchor: DOMRect, fromKeyboard: boolean) => void;
  /** Names the leading toggle. */
  "aria-label"?: string;
  /** Drawn over the toggle's top-right corner, e.g. a warning or a lock. */
  badge?: ReactNode;
  className?: string;
  /** Drawn under the label at the label's width, e.g. a level meter for an
   * audio input that is on. */
  detail?: ReactNode;
  isMenuDisabled?: boolean;
  isSelected?: boolean;
  isToggleDisabled?: boolean;
  onChange?: (isSelected: boolean) => void;
  ref?: Ref<HTMLButtonElement>;
  /** `capture` is the taller control the recording bar carries: the regular
   * glyph and text at the bar's 40px control height. */
  size?: "regular" | "capture";
  /** Named on hover, e.g. what the toggle switches. It hangs off the leading
   * toggle: react-aria hands its trigger props to the nearest focusable, and
   * the whole control carries two. */
  tooltip?: ReactNode;
  /** `ghost` carries no bezel in either state, the toolbar form: the on
   * glyph, a live meter or preview, is what says the input is on. */
  variant?: "filled" | "ghost";
};

const partBase = cn(
  "relative z-10 inline-flex cursor-default items-center justify-center bg-transparent",
  "select-none [&_svg]:shrink-0 [&_svg]:transform-gpu",
  "data-[disabled]:text-control-fg-disabled",
  focusStyles,
  elementFocusVisible,
);

// Without a bezel each part lights up on its own, the way a ghost button
// does; with one, the shared bezel takes the press instead.
const bezellessPart =
  "data-[hovered]:bg-control-subtle-hover data-[pressed]:bg-control-subtle-pressed";

// A Fluent split button lights the part under the pointer on its own bezel,
// so the Windows skin gives each part the bezel's hover and press fills and
// keeps the shared bezel at rest (see the bezel below).
const bezeledPart =
  "windows:data-[hovered]:bg-control-fill-hover windows:data-[pressed]:bg-control-fill-pressed";

const glyphBox = "absolute inset-y-0 flex items-center justify-center";

const scaleAnimation = {
  animate: { opacity: 1, scale: 1 },
  exit: { opacity: 0, scale: 0 },
  initial: { opacity: 0, scale: 0 },
};

const fadeAnimation = {
  animate: { opacity: 1, scale: 1 },
  exit: { opacity: 0, scale: 1 },
  initial: { opacity: 0, scale: 1 },
};

/**
 * A toggle with a pop-up attached: one bezel carrying an on/off switch for an
 * input and, beside it, the device that input is set to. The label is what
 * opens the choices. The Control Center module pattern: two press targets
 * on one bezel.
 */
export function ToggleMenuButton({
  "aria-label": ariaLabel,
  badge,
  children,
  className,
  detail,
  isMenuDisabled = false,
  isSelected = false,
  isToggleDisabled = false,
  label,
  menuLabel,
  off,
  onChange,
  onMenuPress,
  ref,
  size = "regular",
  tooltip,
  variant = "filled",
}: ToggleMenuButtonProps) {
  const isFullyDisabled = isToggleDisabled && isMenuDisabled;
  // A ghost control is the glyph and the device name at any state; being on
  // shows in the glyph itself.
  const hasBezel = variant === "filled";
  const glyphInset =
    size === "capture"
      ? "left-control-inset right-control"
      : "left-control right-tight";

  const toggle = (
    <AriaToggleButton
      aria-label={ariaLabel}
      className={cn(
        partBase,
        "rounded-l-[inherit]",
        "text-content-fg-secondary data-[selected]:text-content-fg",
        "[&_svg.lucide]:size-icon",
        // The glyph keeps its regular size at either height. The part is the
        // glyph plus the control's side inset on the outside and half of
        // the icon-to-label gap on the inside, so the two parts share that
        // gap evenly and each highlights with room around its content.
        size === "capture"
          ? "h-10 w-[calc(var(--spacing-icon)+var(--spacing-control-inset)+var(--spacing-control))]"
          : "h-control-height w-[calc(var(--spacing-icon)+var(--spacing-control)+var(--spacing-tight))]",
        isFullyDisabled ? undefined : hasBezel ? bezeledPart : bezellessPart,
      )}
      isDisabled={isToggleDisabled}
      isSelected={isSelected}
      onChange={onChange}
      ref={ref}
    >
      {({ isSelected: selected }) => (
        // The part is a fixed box, so the two states are stacked in it
        // rather than sized by an invisible copy: the on glyph is a live
        // preview in places, and must be mounted exactly once. The stack
        // sits in the glyph's box, inset from the part's edges as above.
        <AnimatePresence initial={false}>
          {selected ? (
            <motion.span
              key="selected"
              {...scaleAnimation}
              className={cn(glyphBox, glyphInset)}
              exit={fadeAnimation.exit}
            >
              {children}
            </motion.span>
          ) : (
            <motion.span
              key="deselected"
              {...fadeAnimation}
              className={cn(glyphBox, glyphInset)}
            >
              {off}
            </motion.span>
          )}
        </AnimatePresence>
      )}
    </AriaToggleButton>
  );

  return (
    <div
      className={cn(
        "relative isolate inline-flex items-stretch transition",
        size === "capture" ? "rounded-capture" : "rounded-control",
        // The bezel's colours are the platform tokens, as on a button. On
        // macOS a press on either part darkens the whole bezel; a Fluent
        // split button keeps the bezel at rest and lights the pressed part,
        // so the Windows form pins the bezel fill and the parts take over.
        isFullyDisabled
          ? hasBezel
            ? "bg-control-fill-disabled text-control-fg-disabled control-stroke"
            : "bg-transparent text-control-fg-disabled"
          : hasBezel
            ? "bg-control-fill text-content-fg control-stroke has-[[data-pressed]]:bg-control-fill-pressed windows:has-[[data-pressed]]:bg-control-fill"
            : "bg-transparent text-content-fg",
        className,
      )}
      data-toggle-menu-button=""
    >
      <div className="relative flex rounded-l-[inherit]">
        {tooltip ? (
          <TooltipTrigger delay={400}>
            {toggle}
            <Tooltip>{tooltip}</Tooltip>
          </TooltipTrigger>
        ) : (
          toggle
        )}
        {badge ? (
          <span className="pointer-events-none absolute top-0 right-0 z-20 flex [&_svg]:shrink-0 [&_svg.lucide]:size-icon-mini">
            {badge}
          </span>
        ) : null}
      </div>

      <AriaButton
        aria-label={menuLabel}
        className={cn(
          partBase,
          // The label takes the other half of the icon-to-label gap on its
          // inside and the control's side inset on the outside: 2 and 4 at
          // regular, 4 and 8 at capture, matching a button of that height.
          "rounded-r-[inherit]",
          // A toolbar names its items at the small system size, the way
          // Control Center modules do; a form control keeps body text.
          size === "capture"
            ? "h-10 pr-control-inset pl-control text-subheadline"
            : "h-control-height pr-control pl-tight text-body",
          // An input that is off reads quiet, name and all, while staying
          // pressable.
          isSelected ? undefined : "text-content-fg-secondary",
          isFullyDisabled ? undefined : hasBezel ? bezeledPart : bezellessPart,
        )}
        isDisabled={isMenuDisabled}
        // A native pop-up hangs off the whole control, not the label, so the
        // anchor is the whole bezel's bounds.
        onPress={(event) => {
          const part = event.target;
          const bezel = part.closest("[data-toggle-menu-button]") ?? part;
          onMenuPress(
            bezel.getBoundingClientRect(),
            ["keyboard", "virtual"].includes(event.pointerType),
          );
        }}
      >
        {/* Capped rather than fixed: a short device name gives a shorter
            control, a long one is truncated, and the bar carrying it fits
            whichever it is. The detail stretches to the name's width. */}
        <span className="flex max-w-24 flex-col items-stretch gap-tight">
          <span className="truncate text-left">{label}</span>
          {detail}
        </span>
      </AriaButton>
    </div>
  );
}

export type { ToggleMenuButtonProps };
