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

import { cn } from "../../../lib/styling";

type CheckableElementProps = {
  children?: ReactNode;
  className?: string;
  onPress?: (event: PressEvent) => unknown;
};

type CheckOnClickProps = {
  children: ReactElement<CheckableElementProps>;
  onPress: (event: PressEvent) => unknown;
};

/**
 * Adds confirmation feedback to one pressable child without introducing a
 * wrapper element or any button styling: the control's content briefly gives
 * way to a checkmark, as native copy actions confirm themselves. The check
 * lives inside the child, so its dimensions always match that control.
 */
export function CheckOnClick({ children, onPress }: CheckOnClickProps) {
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
            "inline-flex items-center justify-center gap-control-inset transition-opacity",
            status === "pending" && "animate-pulse",
            isChecked && "opacity-0",
          )}
        >
          {children.props.children}
        </span>

        <span className="absolute inset-0 flex items-center justify-center">
          <AnimatePresence>
            {isChecked ? (
              <motion.span
                animate={{ opacity: 1, scale: 1 }}
                className="flex items-center justify-center"
                exit={{ opacity: 0, scale: 0.8 }}
                initial={{ opacity: 0, scale: 0.8 }}
                transition={{ duration: 0.15 }}
              >
                <Check className="size-icon transform-gpu" />
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
