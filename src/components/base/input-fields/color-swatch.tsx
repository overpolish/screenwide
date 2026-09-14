// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { Lock } from "lucide-react";
import { MouseEventHandler, useEffect, useRef, useState } from "react";
import { useFocusRing } from "react-aria";
import { Button } from "react-aria-components";

import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";

/** Tells the wells apart, so only the one that opened the panel hears it. */
let nextToken = 0;

/**
 * Whether the system Colours panel is available. WebKit's own colour popover
 * is a window we can neither place nor theme, so the Mac app asks AppKit for
 * the real panel instead; everywhere else keeps the native input.
 */
function usesSystemColorPanel() {
  return isTauri() && navigator.userAgent.includes("Mac");
}

/**
 * Subscribes to the system Colours panel while `token` owns it, reporting
 * every drag step to `onChange`. Returns the panel to nobody once it closes.
 */
function useColorPanelEvents(
  token: string | null,
  onChange: ((value: string) => void) | undefined,
  onClose: () => void,
) {
  const handlersRef = useRef({ onChange, onClose });
  handlersRef.current = { onChange, onClose };

  useEffect(() => {
    if (token === null) return;
    let listening = true;
    let unlisteners: UnlistenFn[] = [];
    const stop = () => {
      listening = false;
      for (const unlisten of unlisteners) unlisten();
      unlisteners = [];
    };
    void Promise.all([
      listen<{ color: string; token: string }>(
        "color-panel-change",
        (event) => {
          if (event.payload.token !== token) return;
          handlersRef.current.onChange?.(event.payload.color);
        },
      ),
      listen<{ token: string }>("color-panel-close", (event) => {
        if (event.payload.token !== token) return;
        handlersRef.current.onClose();
      }),
    ]).then((registered) => {
      if (!listening) {
        for (const unlisten of registered) unlisten();
        return;
      }
      unlisteners = registered;
    });
    return stop;
  }, [token]);
}

/**
 * A colour well: a square of the colour itself, on the control height and
 * radius its neighbours take. On the Mac app pressing it opens the system
 * Colours panel; everywhere else a native `<input type="color">` sits over it
 * at zero opacity. The well follows that input's keyboard focus visibility.
 *
 * A white colour would otherwise have no edge, so an inset hairline-free ring
 * keeps the well reading as a control whatever it holds.
 */
export function ColorSwatch({
  ariaLabel,
  isDisabled,
  isLocked,
  onChange,
  onContextMenu,
  value,
}: {
  ariaLabel: string;
  value: string;
  isDisabled?: boolean;
  isLocked?: boolean;
  onChange?: (value: string) => void;
  onContextMenu?: MouseEventHandler<HTMLSpanElement>;
}) {
  const systemPanel = usesSystemColorPanel();
  const { focusProps, isFocusVisible } = useFocusRing();
  const [token, setToken] = useState<string | null>(null);
  useColorPanelEvents(token, onChange, () => {
    setToken(null);
  });

  return (
    <span
      className={cn(
        "relative inline-block size-control-height shrink-0 cursor-default overflow-hidden rounded-control align-middle",
        "inset-ring-1 inset-ring-content-fg-quaternary",
        focusStyles,
        "has-[[data-focus-visible]]:ring-3",
        isDisabled && "opacity-50",
      )}
      onContextMenu={onContextMenu}
      style={{ backgroundColor: value }}
    >
      {systemPanel ? (
        <Button
          aria-label={ariaLabel}
          className={cn(
            "absolute inset-0 size-full cursor-default bg-transparent",
            focusStyles,
            elementFocusVisible,
          )}
          isDisabled={isDisabled}
          onPress={() => {
            nextToken += 1;
            const requested = `color-well-${String(nextToken)}`;
            setToken(requested);
            void invoke("show_color_panel", { color: value, token: requested });
          }}
        />
      ) : (
        <input
          {...focusProps}
          aria-label={ariaLabel}
          className="absolute inset-0 size-full cursor-default opacity-0 outline-none"
          data-focus-visible={isFocusVisible || undefined}
          disabled={isDisabled}
          onChange={(event) => {
            onChange?.(event.currentTarget.value.toUpperCase());
          }}
          type="color"
          value={value}
        />
      )}
      {isLocked ? (
        <span className="pointer-events-none absolute inset-0 flex items-center justify-center text-white drop-shadow-[0_0_2px_rgb(0_0_0/0.8)] [&_svg.lucide]:size-icon-mini [&_svg]:shrink-0">
          <Lock aria-hidden="true" />
        </span>
      ) : null}
    </span>
  );
}
