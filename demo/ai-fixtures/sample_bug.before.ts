/**
 * Sample bug for bugfix-agent demo.
 * Off-by-one: tasks render starting one day late.
 */

export function dateToPixel(date: string, startDate: string, dayWidth: number): number {
  const taskDate = new Date(date);
  const originDate = new Date(startDate);
  
  // BUG: Should be taskDate - originDate, not + 1
  const dayOffset = Math.floor(
    (taskDate.getTime() - originDate.getTime()) / (1000 * 60 * 60 * 24)
  ) + 1;  // <- off-by-one here
  
  return dayOffset * dayWidth;
}

export function pixelToDate(pixel: number, startDate: string, dayWidth: number): string {
  const originDate = new Date(startDate);
  const dayOffset = Math.floor(pixel / dayWidth) - 1; // <- matching off-by-one
  const result = new Date(originDate.getTime() + dayOffset * 1000 * 60 * 60 * 24);
  return result.toISOString().split('T')[0];
}
