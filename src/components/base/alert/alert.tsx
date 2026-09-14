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
    // On the layer tokens: the faintest fill on macOS, a Fluent info bar's
    // card fill and hairline stroke on Windows.
    base: "flex items-start gap-control-inset rounded-control bg-layer inset-ring inset-ring-layer-stroke p-section text-body text-content-fg",
    content: "min-w-0",
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
    icon?: ReactNode | false;
  };

export function Alert({
  children,
  className,
  color = "neutral",
  icon: customIcon,
  ...props
}: AlertProps) {
  const { base, content, icon } = alertVariants({
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
    </div>
  );
}
