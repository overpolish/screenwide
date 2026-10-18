// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { clsx } from "clsx";
import {
  ArrowBigUp,
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowRightToLine,
  ArrowUp,
  ChevronUp,
  CircleArrowOutUpLeft,
  Command,
  CornerDownLeft,
  Delete,
  type LucideIcon,
  Option,
  Plus,
} from "lucide-react";
import {
  isValidElement,
  type HTMLAttributes,
  type ReactElement,
  type ReactNode,
  type Ref,
} from "react";

import { isMacHost } from "../../../lib/platform";
import { cn } from "../../../lib/styling";

import { keyLabel } from "./key-label";

// A minimal keycap: a small rounded fill with the key in the label colour,
// as Mac apps draw shortcut hints. No border, bottom edge or shadow; native
// shortcut text is flat, and the fill alone sets the key apart from prose.
// The plain variant drops the fill for keys shown inside a control, where
// native draws the shortcut as bare glyphs.
// `kbd` inherits the monospace family from the browser reset, so the sans
// family is set explicitly. The size is inherited: a key in body text is a
// 20px cap on a 16px line, and one in a subheadline hint shrinks with it,
// so the cap is always a quarter taller than the line it sits in.
const keyboardClassName = {
  // A keycap on the bezel tokens: the plain fill on macOS, Fluent's fill
  // with its elevation stroke on Windows. 1.25 line-heights is 20px on
  // macOS; on Windows it would be 25, an odd height that puts a 12px glyph
  // on a half pixel inside the stroke, so the Windows keycap is 24.
  keycap:
    "inline-flex h-[1.25lh] min-w-[1.25lh] items-center justify-center gap-tight rounded-sm bg-control-fill control-stroke px-control font-sans text-content-fg tabular-nums windows:h-6 windows:min-w-6",
  plain:
    "inline-flex items-center justify-center gap-tight font-sans tabular-nums",
};
const keyboardIconClassName =
  "[&_svg]:size-icon-mini [&_svg]:shrink-0 [&_svg]:transform-gpu";

// A Mac menu draws its modifiers and named keys as symbols; Windows spells
// every key out, as its menus and keycaps do. Keys come in as `hotkeyKeys`
// names them or as a story spells them.
type MacGlyph = {
  Glyph: LucideIcon;
  keys: string[];
  name: string;
  className?: string;
};
const macGlyphList: MacGlyph[] = [
  { Glyph: ArrowBigUp, keys: ["shift", "⇧"], name: "shift" },
  {
    Glyph: Command,
    keys: ["command", "cmd", "⌘", "meta", "super", "win"],
    name: "command",
  },
  { Glyph: ChevronUp, keys: ["control", "ctrl", "⌃"], name: "control" },
  { Glyph: Option, keys: ["option", "alt", "⌥"], name: "option" },
  { Glyph: CornerDownLeft, keys: ["enter", "return"], name: "enter" },
  { Glyph: Delete, keys: ["backspace"], name: "backspace" },
  // Forward delete, ⌦: the backspace symbol turned to point the other way.
  {
    Glyph: Delete,
    className: "-scale-x-100",
    keys: ["delete"],
    name: "delete",
  },
  { Glyph: CircleArrowOutUpLeft, keys: ["esc", "escape"], name: "escape" },
  { Glyph: ArrowRightToLine, keys: ["tab"], name: "tab" },
  { Glyph: ArrowUp, keys: ["up"], name: "up" },
  { Glyph: ArrowDown, keys: ["down"], name: "down" },
  { Glyph: ArrowLeft, keys: ["left"], name: "left" },
  { Glyph: ArrowRight, keys: ["right"], name: "right" },
];
const macGlyphs = new Map(
  macGlyphList.flatMap((glyph) =>
    glyph.keys.map((key) => [key, glyph] as const),
  ),
);

// A symbol is drawn light, beside SF Pro's thin keycap text.
const keyGlyphClassName = "stroke-1";

// Brackets and parentheses drop below the baseline, so SF Pro centres their
// ink about 0.1em below the cap's middle, where letters sit within 0.03em.
// Segoe's metrics differ and its keycap already reads as centred.
const descendingKeys = new Set(["[", "]", "(", ")", "{", "}"]);
const descendingKeyClassName = "-translate-y-[0.1em] windows:translate-y-0";

const mappedKey = (children: ReactNode) => {
  if (typeof children !== "string") return { children };

  const key = children.trim().toLowerCase();
  const glyph = isMacHost ? macGlyphs.get(key) : undefined;
  if (glyph) {
    const { Glyph } = glyph;
    return {
      accessibleName: keyLabel(glyph.name),
      children: (
        <Glyph aria-hidden className={cn(keyGlyphClassName, glyph.className)} />
      ),
    };
  }
  if (descendingKeys.has(key)) {
    return {
      children: <span className={descendingKeyClassName}>{children}</span>,
    };
  }
  return { children: keyLabel(children) };
};

export type KeyboardProps = HTMLAttributes<HTMLElement> & {
  ref?: Ref<HTMLElement>;
  variant?: "keycap" | "plain";
};

export const Keyboard = ({
  "aria-label": ariaLabel,
  children,
  className,
  ref,
  variant = "keycap",
  ...rest
}: KeyboardProps) => {
  const key = mappedKey(children);

  return (
    <kbd
      {...rest}
      aria-label={ariaLabel ?? key.accessibleName}
      className={clsx(
        keyboardClassName[variant],
        keyboardIconClassName,
        className,
      )}
      ref={ref}
    >
      <span className="inline-flex items-center justify-center">
        {key.children}
      </span>
    </kbd>
  );
};

type ShortcutChild = ReactElement<KeyboardProps, typeof Keyboard> | "+";

type ShortcutProps = Omit<HTMLAttributes<HTMLSpanElement>, "children"> & {
  children: ShortcutChild | ShortcutChild[];
  ref?: Ref<HTMLSpanElement>;
};

export const Shortcut = ({
  children,
  className,
  ref,
  ...rest
}: ShortcutProps) => {
  const items = Array.isArray(children) ? children : [children];

  for (const child of items) {
    const isKeyboard = isValidElement(child) && child.type === Keyboard;
    if (!isKeyboard && child !== "+") {
      throw new Error(
        "Shortcut only accepts direct Keyboard elements and literal '+' separators.",
      );
    }
  }

  return (
    <span
      // Keys are named in the order they are pressed, left to right, in
      // every language, as menus show them.
      dir="ltr"
      {...rest}
      className={cn("gap-control inline-flex items-center", className)}
      ref={ref}
    >
      {items.map((child, index) =>
        child === "+" ? (
          <Plus
            aria-hidden
            className="size-icon-mini shrink-0 transform-gpu text-content-fg-secondary"
            key={`separator-${index.toString()}`}
          />
        ) : (
          child
        ),
      )}
    </span>
  );
};
