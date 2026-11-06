// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

/** What one product's Payhip check says about a key. */
export type ProductAnswer =
  | { kind: "enabled"; email: string | undefined }
  | { kind: "disabled" }
  /** Payhip does not know the key for this product. */
  | { kind: "unknown" }
  /** No answer to rely on: a network error, a rate limit, Payhip failing,
   * or a product secret Payhip rejects. */
  | { kind: "failed" };

const VERIFY = "https://payhip.com/api/v2/license/verify";

/**
 * Asks Payhip whether `licenseKey` is one of a product's keys. A key is only
 * "unknown" on a clear answer: Payhip's reply for a key it does not have, a
 * 400 of `{"data":[],"error":true}` (seen live), or a success without a
 * licence. A rejected product secret is a 401 (`authentication_fail`), and
 * reading that as "unknown" would make every re-check take away every buyer's
 * key, so it and any other error count as "failed", which the app reads as no
 * answer.
 */
export async function checkProduct(
  licenseKey: string,
  productSecret: string,
  fetcher: typeof fetch,
): Promise<ProductAnswer> {
  let response: Response;
  try {
    response = await fetcher(
      `${VERIFY}?license_key=${encodeURIComponent(licenseKey)}`,
      { headers: { "product-secret-key": productSecret } },
    );
  } catch {
    return { kind: "failed" };
  }
  if (response.status === 404) return { kind: "unknown" };
  if (response.status === 400) {
    return (await isNoLicence(response))
      ? { kind: "unknown" }
      : { kind: "failed" };
  }
  if (!response.ok) return { kind: "failed" };

  const text = await response.text();
  if (text.trim() === "") return { kind: "unknown" };
  let body: unknown;
  try {
    body = JSON.parse(text);
  } catch {
    return { kind: "failed" };
  }
  const data =
    typeof body === "object" && body !== null && "data" in body
      ? body.data
      : undefined;
  if (typeof data !== "object" || data === null || !("enabled" in data)) {
    return { kind: "unknown" };
  }
  if (!isTrue(data.enabled)) return { kind: "disabled" };
  const email =
    "buyer_email" in data && typeof data.buyer_email === "string"
      ? data.buyer_email
      : undefined;
  return { kind: "enabled", email };
}

/** Payhip's answer for a key the product does not have: an error with no
 * licence data. */
async function isNoLicence(response: Response): Promise<boolean> {
  try {
    const body: unknown = await response.json();
    return (
      typeof body === "object" &&
      body !== null &&
      "error" in body &&
      body.error === true &&
      "data" in body &&
      Array.isArray(body.data) &&
      body.data.length === 0
    );
  } catch {
    return false;
  }
}

// Payhip has sent the flag as a boolean, a number and a string.
function isTrue(value: unknown): boolean {
  return value === true || value === 1 || value === "true" || value === "1";
}
