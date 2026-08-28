// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import {
  cloneElement,
  isValidElement,
  ReactElement,
  ReactNode,
  useEffect,
  useRef,
  useState,
} from "react";
import { PressEvent } from "react-aria";
import { VariantProps } from "tailwind-variants";

import { availableVariants, cn } from "../../../lib/styling";
import { tv } from "../../../lib/variants";

const checkOnClickOverlay = tv({
  base: "absolute inset-0 flex items-center justify-center rounded-[inherit] transition-all backdrop-blur-none",
  compoundVariants: [
    {
      blur: "md",
      class: "backdrop-blur-md",
      isChecked: true,
    },
    {
      blur: "xs",
      class: "backdrop-blur-xs",
      isChecked: true,
    },
  ],
  variants: {
    blur: availableVariants("md", "xs"),
    isChecked: {
      true: "bg-content/50",
    },
  },
});

type CheckableElementProps = {
  children?: ReactNode;
  className?: string;
  onPress?: (event: PressEvent) => unknown;
};

type CheckOnClickProps = VariantProps<typeof checkOnClickOverlay> & {
  children: ReactElement<CheckableElementProps>;
  onPress: (event: PressEvent) => unknown;
};

/**
 * Adds confirmation feedback to one pressable child without introducing a
 * wrapper element or any button styling. The overlay lives inside the child,
 * so its dimensions and inherited radius always match that control.
 */
export function CheckOnClick({
  blur = "md",
  children,
  onPress,
}: CheckOnClickProps) {
  const [status, setStatus] = useState<"checked" | "idle" | "pending">("idle");
  const pressTokenRef = useRef(0);
  const isMountedRef = useRef(false);

  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  if (!isValidElement<CheckableElementProps>(children)) return null;

  const handlePress = (event: PressEvent) => {
    if (status !== "idle") return;

    const token = ++pressTokenRef.current;
    const isCurrent = () =>
      isMountedRef.current && pressTokenRef.current === token;
    const showCheck = () => {
      if (!isCurrent()) return;
      setStatus("checked");
      setTimeout(() => {
        if (isCurrent()) setStatus("idle");
      }, 2000);
    };

    const result = onPress(event);
    if (result instanceof Promise) {
      setStatus("pending");
      void result.then(showCheck, () => {
        if (isCurrent()) setStatus("idle");
      });
      return;
    }

    showCheck();
  };

  const isChecked = status === "checked";

  // Cloning is deliberate here: injecting the overlay into the actual
  // control lets it inherit that element's radius without adding a sizing or
  // focusable wrapper of its own.
  // eslint-disable-next-line @eslint-react/no-clone-element
  return cloneElement(children, {
    children: (
      <>
        <span
          className={cn(
            "inline-flex items-center justify-center gap-2",
            status === "pending" && "animate-pulse",
          )}
        >
          {children.props.children}
        </span>

        <span className={checkOnClickOverlay({ blur, isChecked })}>
          <AnimatePresence>
            {isChecked ? (
              <motion.span
                animate={{ opacity: 1, y: 0 }}
                className="flex h-full items-center justify-center"
                exit={{ opacity: 0, y: -5 }}
                initial={{ opacity: 0, y: 5 }}
                transition={{ duration: 0.2 }}
              >
                <Check className="h-1/2 w-auto text-success" strokeWidth={3} />
              </motion.span>
            ) : null}
          </AnimatePresence>
        </span>
      </>
    ),
    className: cn(
      "relative",
      children.props.className,
      status !== "idle" && "pointer-events-none",
    ),
    onPress: handlePress,
  });
}

export type { CheckOnClickProps };
