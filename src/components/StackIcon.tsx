/** A stack: two partly overlapping photos (F015/S03). */
export default function StackIcon({ size = 16, open = false }: { size?: number; open?: boolean }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}
      strokeLinejoin="round" aria-hidden="true">
      <rect x="7" y="3" width="14" height="12" rx="2" fill={open ? "none" : "currentColor"} fillOpacity={0.25} />
      <rect x="3" y="9" width="14" height="12" rx="2" fill="white" fillOpacity={0.9} />
      <path d="M5.5 18.5l3.5-4 2.5 2.5 2-2 2 3.5" strokeWidth={1.6} strokeLinecap="round" />
    </svg>
  );
}
