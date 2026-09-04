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

const keyboardClassName = [
  "gap-tight inline-flex min-h-5 min-w-5 items-center justify-center rounded-md border border-content-fg/20 border-b-2 border-b-content-fg/35 bg-content-fg/[7%] px-control font-mono text-xs leading-none text-content-fg shadow-xs tabular-nums",
  "inverse:border-content/20 inverse:border-b-content/35 inverse:bg-content/[7%] inverse:text-content",
  "[&_svg]:size-3 [&_svg]:shrink-0 [&_svg]:transform-gpu",
];

const isMacOS =
  typeof navigator !== "undefined" && navigator.userAgent.includes("Mac");

const mappedKey = (children: ReactNode) => {
  if (typeof children !== "string") return { children };

  const key = children.trim().toLowerCase();
  if (key === "shift" || key === "⇧") {
    return {
      accessibleName: "Shift",
      children: <ArrowBigUp aria-hidden />,
    };
  }
  if (key === "command" || key === "cmd" || key === "⌘") {
    return {
      accessibleName: "Command",
      children: <Command aria-hidden />,
    };
  }
  if (key === "meta" || key === "super") {
    return isMacOS
      ? {
          accessibleName: "Command",
          children: <Command aria-hidden />,
        }
      : { children: "Win" };
  }
  if (key === "control" || key === "ctrl" || key === "⌃") {
    return isMacOS
      ? {
          accessibleName: "Control",
          children: <ChevronUp aria-hidden />,
        }
      : { children: "Ctrl" };
  }
  return { children };
};

export type KeyboardProps = HTMLAttributes<HTMLElement> & {
  ref?: Ref<HTMLElement>;
};

export const Keyboard = ({
  "aria-label": ariaLabel,
  children,
  className,
  ref,
  ...rest
}: KeyboardProps) => {
  const key = mappedKey(children);

  return (
    <kbd
      {...rest}
      aria-label={ariaLabel ?? key.accessibleName}
      className={clsx(keyboardClassName, className)}
      ref={ref}
    >
      <span className="inline-flex -translate-y-px transform-gpu items-center justify-center">
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
            className="size-3 shrink-0 transform-gpu"
            key={`separator-${index.toString()}`}
          />
        ) : (
          child
        ),
      )}
    </span>
  );
};
