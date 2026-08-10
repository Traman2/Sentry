export function healthColor(percent: number) {
  if (percent >= 50) return "var(--color-danger)";
  if (percent >= 15) return "#c9a227";
  return "#3fa66b";
}
