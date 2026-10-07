/** Formats an ISO timestamp as a compact relative age such as "5m ago". */
export function timeAgo(value: string, now: number = Date.now()): string {
  const seconds = Math.max(1, Math.floor((now - new Date(value).getTime()) / 1000));
  if (seconds < 60) return `${seconds}s ago`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
}
