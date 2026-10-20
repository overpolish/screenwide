// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Info, OctagonAlert } from "lucide-react";
import { ComponentProps, ReactNode } from "react";
import { VariantProps } from "tailwind-variants";

import { tv } from "../../../lib/variants";

// An inline notice: the translucent fill on the window's material, the
// message in the text colour, and an icon that carries the meaning. A status
// colours the icon alone; the box and the text are never tinted.
const alertVariants = tv({
  defaultVariants: {
    color: "neutral",
  },
  slots: {
    // Centred on the first line, as the icon is: the slot is a line tall and
    // the control overhangs it evenly into the padding.
    action: "flex h-[1lh] shrink-0 items-center",
    base: "flex items-start gap-separate rounded-panel bg-fill p-inset text-body text-fg",
    content: "min-w-0 grow",
    icon: "flex h-[1lh] shrink-0 items-center [&>svg]:size-icon [&>svg]:transform-gpu",
    // The icon and the message, read as one.
    lead: "flex min-w-0 grow items-start gap-inline",
  },
  variants: {
    color: {
      error: {
        icon: "text-error",
      },
      neutral: {
        icon: "text-fg-secondary",
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
    /** Replaces the default icon; `false` hides it. */
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
  const { action, base, content, icon, lead } = alertVariants({
    className,
    color,
  });
  const DefaultIcon = defaultIcons[color];

  return (
    <div className={base()} {...props}>
      <div className={lead()}>
        {customIcon === false ? null : (
          <span aria-hidden="true" className={icon()}>
            {customIcon ?? <DefaultIcon />}
          </span>
        )}
        <div className={content()}>{children}</div>
      </div>
      {actionContent ? <div className={action()}>{actionContent}</div> : null}
    </div>
  );
}
