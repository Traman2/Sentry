export interface SeriesStats {
  /** Most recent sampled value in the window. */
  current: number;
  average: number;
  peak: number;
}

const EMPTY: SeriesStats = { current: 0, average: 0, peak: 0 };

/** Summary figures for one charted line, shown as the stat cards above the
 * chart. Assumes `values` is in chronological order, as the series builders
 * in `series.ts` produce. */
export function computeSeriesStats(values: number[]): SeriesStats {
  if (values.length === 0) return EMPTY;

  let sum = 0;
  let peak = 0;
  for (const value of values) {
    sum += value;
    if (value > peak) peak = value;
  }

  return { current: values[values.length - 1], average: sum / values.length, peak };
}
