// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Info, OctagonAlert } from "lucide-react";
import { ComponentProps, ReactNode } from "react";
import { VariantProps } from "tailwind-variants";

import { tv } from "../../../lib/variants";

const alertVariants = tv({
  defaultVariants: {
    color: "neutral",
  },
  slots: {
    base: "gap-control-inset p-section flex items-start rounded-xl text-sm text-content-fg",
    content: "min-w-0 leading-normal",
    icon:
      "flex h-[1lh] w-icon-default shrink-0 items-center justify-center [&>svg]:size-icon-default",
  },
  variants: {
    color: {
      error: {
        base: "bg-error-surface text-error",
        icon: "text-error",
      },
      neutral: {
        base: "bg-neutral",
        icon: "text-muted",
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
