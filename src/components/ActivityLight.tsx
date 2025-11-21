import { motion, AnimatePresence } from 'framer-motion';

export interface ActivityLightProps {
  /** Whether the activity light is currently active */
  active: boolean;
  /** Color of the activity light (Tailwind class) */
  color?: 'emerald' | 'cyan' | 'amber';
  /** Size of the light in pixels */
  size?: number;
  /** Accessible label for screen readers */
  ariaLabel?: string;
}

/**
 * ActivityLight component shows when a parameter is receiving data.
 * Fades out after 500ms of inactivity.
 */
export function ActivityLight({
  active,
  color = 'emerald',
  size = 8,
  ariaLabel = 'Activity indicator',
}: ActivityLightProps) {
  const colorClasses = {
    emerald: 'bg-emerald-500 shadow-emerald-500/50',
    cyan: 'bg-cyan-500 shadow-cyan-500/50',
    amber: 'bg-amber-500 shadow-amber-500/50',
  };

  return (
    <div
      className="flex items-center justify-center"
      role="status"
      aria-label={ariaLabel}
      aria-live="polite"
    >
      <AnimatePresence>
        {active && (
          <motion.div
            initial={{ opacity: 0, scale: 0.5 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.5 }}
            transition={{ duration: 0.15 }}
            className={`rounded-full ${colorClasses[color]}`}
            style={{
              width: size,
              height: size,
              boxShadow: `0 0 ${size}px currentColor`,
            }}
          />
        )}
      </AnimatePresence>
    </div>
  );
}
