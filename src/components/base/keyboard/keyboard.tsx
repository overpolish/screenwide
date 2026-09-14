// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { clsx } from "clsx";
import { ArrowBigUp, ChevronUp, Command, Plus } from "lucide-react";
import {
  isValidElement,
  type HTMLAttributes,
  type ReactElement,
  type ReactNode,
  type Ref,
} from "react";

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

// A modifier glyph is drawn light beside SF Pro's thin keycap text; Segoe is
// heavier, so on Windows the glyph keeps the icon set's regular stroke.
const keyGlyphClassName = "stroke-1 windows:stroke-[1.5px]";

const isMacOS =
  typeof navigator !== "undefined" && navigator.userAgent.includes("Mac");

const mappedKey = (children: ReactNode) => {
  if (typeof children !== "string") return { children };

  const key = children.trim().toLowerCase();
  if (key === "shift" || key === "⇧") {
    return {
      accessibleName: "Shift",
      children: <ArrowBigUp aria-hidden className={keyGlyphClassName} />,
    };
  }
  if (key === "command" || key === "cmd" || key === "⌘") {
    return {
      accessibleName: "Command",
      children: <Command aria-hidden className={keyGlyphClassName} />,
    };
  }
  if (key === "meta" || key === "super") {
    return isMacOS
      ? {
          accessibleName: "Command",
          children: <Command aria-hidden className={keyGlyphClassName} />,
        }
      : { children: "Win" };
  }
  if (key === "control" || key === "ctrl" || key === "⌃") {
    return isMacOS
      ? {
          accessibleName: "Control",
          children: <ChevronUp aria-hidden className={keyGlyphClassName} />,
        }
      : { children: "Ctrl" };
  }
  return { children };
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
      {...rest}
      className={clsx("gap-control inline-flex items-center", className)}
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
