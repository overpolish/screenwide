// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** The stage's camera: light from one side, so a flip can be seen. The real
 * preview mirrors itself as it is set. */
export function SamplePicture({ isFlipped }: { isFlipped: boolean }) {
  return (
    <span
      className="block size-full bg-[radial-gradient(circle_at_28%_35%,#e3bf98,#5a4636_65%)]"
      style={{ transform: isFlipped ? "scaleX(-1)" : undefined }}
    />
  );
}
