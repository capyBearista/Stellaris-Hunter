export function IconPlaceholder({ size = 32 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
      <rect width="32" height="32" rx="4" fill="#2a2a3a" />
      <path d="M16 8l2.5 5 5.5.8-4 3.9.9 5.3L16 20.5 11.1 23l.9-5.3-4-3.9 5.5-.8L16 8z" fill="#555" />
    </svg>
  );
}
