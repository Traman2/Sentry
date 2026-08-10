import { healthColor } from "@/lib/health";

function Gauge({ percent, size = 12, strokeWidth = 2 }: { percent: number; size?: number; strokeWidth?: number }) {
  const clamped = Math.min(Math.max(percent, 0), 100);
  const color = healthColor(percent);
  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  const dash = (clamped / 100) * circumference;

  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} className="-rotate-90 shrink-0">
      <circle
        cx={size / 2}
        cy={size / 2}
        r={radius}
        fill="none"
        stroke="rgba(38,38,79,0.15)"
        strokeWidth={strokeWidth}
      />
      <circle
        cx={size / 2}
        cy={size / 2}
        r={radius}
        fill="none"
        stroke={color}
        strokeWidth={strokeWidth}
        strokeLinecap="round"
        strokeDasharray={`${dash} ${circumference}`}
      />
    </svg>
  );
}

export default Gauge;
