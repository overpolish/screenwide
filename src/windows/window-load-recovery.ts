// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** How many times this window has reloaded for code it could not load. */
const ATTEMPTS_KEY = "screenwide-window-load-attempts";
/** Past this, the window stays as it is rather than reload without end. */
const MAX_RELOADS = 3;

const attempts = () => {
  try {
    return Number(sessionStorage.getItem(ATTEMPTS_KEY) ?? 0);
  } catch {
    return MAX_RELOADS;
  }
};

/**
 * Answers a window whose own code failed to load: the failure is reported,
 * and the page loads again a moment later, a few times at most. A failed
 * import is remembered for the page's life, so asking again would only fail
 * again; and a window that never loaded its code ignores everything sent to
 * it, so without this it stays blank until the app is quit. At launch every
 * window loads at once, which is when a load is most likely to fail.
 */
export function recoverFromFailedLoad(cause: unknown) {
  console.error("Could not load this window's code", cause);
  const tried = attempts();
  if (tried >= MAX_RELOADS) return;
  try {
    sessionStorage.setItem(ATTEMPTS_KEY, String(tried + 1));
  } catch {
    return;
  }
  window.setTimeout(
    () => {
      window.location.reload();
    },
    250 * (tried + 1),
  );
}

/** Clears the count once the window's code has loaded. */
export function forgetFailedLoads() {
  try {
    sessionStorage.removeItem(ATTEMPTS_KEY);
  } catch {
    // Nothing kept, nothing to clear.
  }
}
