// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { type HTMLAttributes, useRef } from "react";
import { useOverlay } from "react-aria";
import { VariantProps } from "tailwind-variants";

import { motionDurations, motionEasings } from "../../../lib/motion";
import { tv } from "../../../lib/variants";

const overlayVariants = tv({
  base: "fixed inset-0 z-50 flex items-center justify-center bg-overlay text-content-fg",
  defaultVariants: {
    blur: "sm",
  },
  variants: {
    blur: {
      lg: "backdrop-blur-lg",
      md: "backdrop-blur-md",
      sm: "backdrop-blur-sm",
      xs: "backdrop-blur-xs",
    },
    contained: {
      true: "absolute",
    },
  },
});

export type OverlayProps = HTMLAttributes<HTMLDivElement> &
  VariantProps<typeof overlayVariants> & {
    children?: React.ReactNode;
    className?: string;
    isOpen?: boolean;
  };

export const Overlay = ({
  blur,
  children,
  className,
  contained,
  isOpen,
  ...props
}: OverlayProps) => {
  const ref = useRef<HTMLDivElement>(null);
  const { overlayProps } = useOverlay(
    {
      isDismissable: false,
      isKeyboardDismissDisabled: true,
      isOpen: isOpen ?? false,
    },
    ref,
  );

  // Stays mounted through its exit so the blur fades out as it faded in.
  return (
    <AnimatePresence>
      {isOpen ? (
        <motion.div
          animate={{ opacity: 1 }}
          className={overlayVariants({ blur, className, contained })}
          exit={{ opacity: 0 }}
          initial={{ opacity: 0 }}
          transition={{
            duration: motionDurations.travel,
            ease: motionEasings.out,
          }}
        >
          <div {...overlayProps} {...props} className="contents" ref={ref}>
            {children}
          </div>
        </motion.div>
      ) : null}
    </AnimatePresence>
  );
};
