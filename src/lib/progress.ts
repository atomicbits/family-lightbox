/** How far, from 0 to 1, or null when the total isn't known yet. */
export function fraction(done: number | undefined, total: number | undefined): number | null {
  if (done === undefined || !total || total <= 0) return null;
  return Math.min(1, Math.max(0, done / total));
}
