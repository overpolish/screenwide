// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { clickAt, cursorAt } from "./cursor-path";

// The recording's own pointer, with a ripple where it clicked unless the
// clicks are not shown.
export function RecordedCursor({
  hasRipple = true,
  seconds,
}: {
  seconds: number;
  hasRipple?: boolean;
}) {
  const { x, y } = cursorAt(seconds);
  const ripple = hasRipple ? clickAt(seconds) : null;
  return (
    <span
      className="pointer-events-none absolute"
      style={{ insetInlineStart: `${String(x)}%`, top: `${String(y)}%` }}
    >
      {ripple === null ? null : (
        <span
          className="absolute size-10 -translate-1/2 rounded-full border-2 border-[#0a84ff]"
          style={{
            opacity: 1 - ripple,
            transform: `translate(-50%, -50%) scale(${String(0.4 + ripple)})`,
          }}
        />
      )}
      <svg
        aria-hidden
        className="absolute -top-0.5 -translate-x-0.5"
        height={20}
        viewBox="0 0 16 20"
        width={16}
      >
        <path
          d="M1 1 L1 16 L5 12 L8 19 L11 18 L8 11 L14 11 Z"
          fill="#000"
          stroke="#fff"
          strokeWidth={1.2}
        />
      </svg>
    </span>
  );
}
