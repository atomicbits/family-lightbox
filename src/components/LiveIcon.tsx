/** A Live Photo's video: three concentric circles, white with a soft shadow like the camera (F017/S02). */
export default function LiveIcon({ size = 14 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="white" aria-hidden="true"
      style={{ filter: "drop-shadow(0 0 1.5px rgba(0,0,0,0.8))" }}>
      <circle cx="12" cy="12" r="10" strokeWidth={1.6} strokeDasharray="1.6 2.2" />
      <circle cx="12" cy="12" r="6" strokeWidth={2} />
      <circle cx="12" cy="12" r="2.5" fill="white" stroke="none" />
    </svg>
  );
}
