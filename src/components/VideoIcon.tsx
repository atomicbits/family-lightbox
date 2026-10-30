/** A video: a small camera, white with a soft shadow so it reads on any photo. */
export default function VideoIcon({ size = 14 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="white" aria-hidden="true"
      style={{ filter: "drop-shadow(0 0 1.5px rgba(0,0,0,0.8))" }}>
      <rect x="2" y="6" width="14" height="12" rx="2.5" />
      <path d="M17 10.5l5-3v9l-5-3z" />
    </svg>
  );
}
