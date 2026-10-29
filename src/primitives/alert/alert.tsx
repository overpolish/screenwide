// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Info, OctagonAlert } from "lucide-react";
import { ComponentProps, ReactNode } from "react";
import { VariantProps } from "tailwind-variants";

import { tv } from "../../lib/variants";
import { useSkin } from "../skin/skin";
import { Text } from "../text/text";

// An inline notice: the translucent fill on the window's material, the
// message in the text colour, and an icon that carries the meaning. A status
// colours the icon alone; the box and the text are never tinted. On the
// island it is a line of the island's black, the box being the island, and
// speaks as the island's hints do: a caption, set back, centred under the
// parts it is about. One with an action keeps to the start, so the message
// and its answer read across the line.
const alertVariants = tv({
  compoundVariants: [
    {
      class: { base: "justify-center text-center", lead: "grow-0" },
      hasAction: false,
      skin: "island",
    },
  ],
  defaultVariants: {
    color: "neutral",
    skin: "window",
  },
  slots: {
    // Centred on the first line, as the icon is: the slot is a line tall and
    // the control overhangs it evenly into the padding.
    action: "flex h-[1lh] shrink-0 items-center",
    base: "flex items-start gap-separate rounded-panel bg-fill p-inset",
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
    hasAction: {
      false: {},
      true: {},
    },
    skin: {
      island: {
        base: "items-center gap-inline rounded-none bg-transparent p-0 text-caption text-fg-secondary",
        icon: "[&>svg]:size-icon-sm",
      },
      window: {},
    },
  },
});

const defaultIcons = {
  error: OctagonAlert,
  neutral: Info,
};

type AlertProps = Omit<ComponentProps<"div">, "color"> &
  Omit<VariantProps<typeof alertVariants>, "hasAction" | "skin"> & {
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
    color,
    hasAction: Boolean(actionContent),
    skin: useSkin(),
  });
  const DefaultIcon = defaultIcons[color];

  // The icon and action slots, a line of the copy tall, centre on the
  // message's first line.
  return (
    // A slotted variant takes a caller's class on the slot it styles.
    <Text {...props} as="div" className={base({ className })}>
      <div className={lead()}>
        {customIcon === false ? null : (
          <span aria-hidden="true" className={icon()}>
            {customIcon ?? <DefaultIcon />}
          </span>
        )}
        <div className={content()}>{children}</div>
      </div>
      {actionContent ? <div className={action()}>{actionContent}</div> : null}
    </Text>
  );
}
