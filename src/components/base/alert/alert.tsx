// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Info, OctagonAlert } from "lucide-react";
import { ComponentProps, ReactNode } from "react";
import { VariantProps } from "tailwind-variants";

import { tv } from "../../../lib/variants";

// An inline notice, as native apps show one: a grouped box with a faint fill,
// body text in the label colour, and a symbol that carries the meaning. The
// text is never tinted; colour is applied to the glyph alone.
const alertVariants = tv({
  defaultVariants: {
    color: "neutral",
  },
  slots: {
    // A Fluent info bar's action sits at its trailing edge, centred on the
    // first line the way the icon is: the slot is a line tall and the
    // control overhangs it evenly into the padding.
    action: "flex h-[1lh] shrink-0 items-center",
    // On the layer tokens: the faintest fill on macOS, a Fluent info bar's
    // card fill and hairline stroke on Windows.
    base: "flex items-start gap-control-inset rounded-control bg-layer inset-ring inset-ring-layer-stroke p-section text-body text-content-fg",
    content: "min-w-0 grow",
    icon: "flex h-[1lh] w-icon shrink-0 items-center justify-center [&>svg]:size-icon [&>svg]:transform-gpu",
  },
  variants: {
    color: {
      error: {
        icon: "text-error",
      },
      neutral: {
        icon: "text-content-fg-secondary",
      },
    },
  },
});

const defaultIcons = {
  error: OctagonAlert,
  neutral: Info,
};

type AlertProps = Omit<ComponentProps<"div">, "color"> &
  VariantProps<typeof alertVariants> & {
    /** A control that answers the notice, such as Undo. */
    action?: ReactNode;
    icon?: ReactNode | false;
  };

export function Alert({
  action: actionContent,
  children,
  className,
  color = "neutral",
  icon: customIcon,
  ...props
}: AlertProps) {
  const { action, base, content, icon } = alertVariants({
    className,
    color,
  });
  const DefaultIcon = defaultIcons[color];

  return (
    <div className={base()} {...props}>
      {customIcon === false ? null : (
        <span aria-hidden="true" className={icon()}>
          {customIcon ?? <DefaultIcon />}
        </span>
      )}
      <div className={content()}>{children}</div>
      {actionContent ? <div className={action()}>{actionContent}</div> : null}
    </div>
  );
}
