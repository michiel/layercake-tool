export interface Rgb {
  r: number
  g: number
  b: number
}

/**
 * Parse a hex colour string into RGB components.
 *
 * Handles both shorthand (`#abc`) and full (`#aabbcc`) hex, with or without the
 * leading `#`. Returns black for anything that isn't a valid 3- or 6-digit hex,
 * rather than producing garbage from `parseInt` bit-shifting (e.g. `#abc` was
 * previously parsed as the number 2748, yielding wrong R/G/B).
 */
export function hexToRgb(value: string | null | undefined): Rgb {
  const fallback: Rgb = { r: 0, g: 0, b: 0 }
  if (!value) return fallback

  let hex = value.trim().replace(/^#/, '')

  // Expand shorthand #abc -> #aabbcc
  if (hex.length === 3) {
    hex = hex
      .split('')
      .map((c) => c + c)
      .join('')
  }

  if (hex.length !== 6 || !/^[0-9a-fA-F]{6}$/.test(hex)) {
    return fallback
  }

  const int = parseInt(hex, 16)
  return {
    r: (int >> 16) & 255,
    g: (int >> 8) & 255,
    b: int & 255,
  }
}
