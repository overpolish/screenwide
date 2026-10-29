// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
  type HTMLAttributes,
  type ReactElement,
  type ReactNode,
  type Ref,
} from "react";

import { isMacHost } from "../../lib/platform";
import { cn } from "../../lib/styling";

import { keyLabel } from "./key-label";

// A key is caption text in the secondary colour, its symbols the extra-small
// icon, whose thinner stroke matches the caption's weight; `kbd` inherits
// the monospace family from the browser reset, so the sans family is set
// explicitly. A keycap is a small part, drawn as a badge is: that text on
// the faint fill, flat, as shortcut hints are. The plain variant is the key
// without the fill, for a shortcut inside a control or a menu row, where a
// fill would read as a second control; it sits beside the row's label and
// glyph, so it takes their sizes, as a Mac menu sets its keys.
const keyClassName =
  "inline-flex items-center justify-center gap-inline font-sans text-caption text-fg-secondary tabular-nums [&_svg]:size-icon-xs [&_svg]:shrink-0 [&_svg]:transform-gpu";
const keyboardClassName = {
  keycap: "h-part-height min-w-part-height rounded-part bg-fill px-inset-sm",
  plain: "text-body [&_svg]:size-icon-sm",
};

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

const mappedKey = (children: ReactNode) => {
  if (typeof children !== "string") return { children };

  const key = children.trim().toLowerCase();
  const glyph = isMacHost ? macGlyphs.get(key) : undefined;
  if (glyph) {
    const { Glyph } = glyph;
    return {
      accessibleName: keyLabel(glyph.name),
      children: <Glyph aria-hidden className={glyph.className} />,
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
      className={cn(keyClassName, keyboardClassName[variant], className)}
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
  // The children's type limits them to keys and '+' separators. There is no
  // runtime check against `Keyboard` itself: a hot reload replaces that
  // function while elements made before it still point at the old one, so a
  // check by identity throws on correct children until the page reloads.
  const items = Array.isArray(children) ? children : [children];

  return (
    <span
      // Keys are named in the order they are pressed, left to right, in
      // every language, as menus show them.
      dir="ltr"
      {...rest}
      className={cn("inline-flex items-center gap-inline", className)}
      ref={ref}
    >
      {items.map((child, index) =>
        child === "+" ? (
          <Plus
            aria-hidden
            className="size-icon-xs shrink-0 transform-gpu text-fg-secondary"
            key={`separator-${index.toString()}`}
          />
        ) : (
          child
        ),
      )}
    </span>
  );
};
