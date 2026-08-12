export function healthColor(percent: number) {
  if (percent >= 50) return "var(--color-danger)";
  if (percent >= 15) return "var(--color-warning)";
  return "var(--color-success)";
}
