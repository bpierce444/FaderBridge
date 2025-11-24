/**
 * SyncStatusIndicator component
 * Displays the current sync status and latency statistics
 */

import { useActiveSync } from '../hooks/useActiveSync';

export interface SyncStatusIndicatorProps {
  /** Whether to show detailed stats */
  showDetails?: boolean;
  /** Optional CSS class name */
  className?: string;
}

/**
 * Component for displaying sync status and latency
 * 
 * @example
 * ```tsx
 * <SyncStatusIndicator showDetails={true} />
 * ```
 */
export function SyncStatusIndicator({
  showDetails = false,
  className = '',
}: SyncStatusIndicatorProps) {
  const { status, loading, error, startSync, stopSync, clearLatencyStats } = useActiveSync();

  const handleToggleSync = async () => {
    if (status.active) {
      await stopSync();
    } else {
      await startSync();
    }
  };

  // Determine status color and text
  const getStatusInfo = () => {
    if (!status.initialized) {
      return {
        color: 'bg-slate-600',
        text: 'Not Initialized',
        glowColor: '',
      };
    }
    if (!status.active || status.mappingCount === 0) {
      return {
        color: 'bg-slate-600',
        text: 'Inactive',
        glowColor: '',
      };
    }

    // Check latency
    const avgLatency = status.latencyStats?.avg_ms ?? 0;
    if (avgLatency === 0) {
      return {
        color: 'bg-cyan-400',
        text: 'Active',
        glowColor: 'shadow-lg shadow-cyan-500/50',
      };
    } else if (avgLatency < 10) {
      return {
        color: 'bg-emerald-500',
        text: 'Active',
        glowColor: 'shadow-lg shadow-emerald-500/50',
      };
    } else if (avgLatency < 20) {
      return {
        color: 'bg-amber-500',
        text: 'Active (Slow)',
        glowColor: 'shadow-lg shadow-amber-500/50',
      };
    } else {
      return {
        color: 'bg-rose-600',
        text: 'Active (Lagging)',
        glowColor: 'shadow-lg shadow-rose-600/50',
      };
    }
  };

  const statusInfo = getStatusInfo();

  if (loading) {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <div className="w-3 h-3 rounded-full bg-slate-600 animate-pulse"></div>
        <span className="text-sm text-slate-400">Loading...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <div className="w-3 h-3 rounded-full bg-rose-600"></div>
        <span className="text-sm text-rose-400">Error: {error}</span>
      </div>
    );
  }

  return (
    <div className={`space-y-3 ${className}`}>
      {/* Status Indicator */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className={`w-3 h-3 rounded-full ${statusInfo.color} ${statusInfo.glowColor}`}></div>
          <div>
            <div className="text-sm font-semibold text-white">{statusInfo.text}</div>
            {status.mappingCount > 0 && (
              <div className="text-xs text-slate-400">
                {status.mappingCount} {status.mappingCount === 1 ? 'mapping' : 'mappings'}
              </div>
            )}
          </div>
        </div>

        {/* Toggle Button */}
        {status.initialized && status.mappingCount > 0 && (
          <button
            onClick={handleToggleSync}
            className={`px-3 py-1 text-sm font-semibold rounded transition-colors ${
              status.active
                ? 'bg-slate-700 hover:bg-slate-600 text-white'
                : 'bg-cyan-500 hover:bg-cyan-600 text-slate-950'
            }`}
            aria-label={status.active ? 'Stop sync' : 'Start sync'}
          >
            {status.active ? 'Stop' : 'Start'}
          </button>
        )}
      </div>

      {/* Detailed Stats */}
      {showDetails && status.latencyStats && status.latencyStats.sample_count > 0 && (
        <div className="p-3 bg-slate-900 border border-slate-800 rounded space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-sm font-semibold text-slate-300">Latency Statistics</span>
            <button
              onClick={clearLatencyStats}
              className="text-xs text-cyan-400 hover:text-cyan-300"
              aria-label="Clear latency stats"
            >
              Clear
            </button>
          </div>

          <div className="grid grid-cols-3 gap-3 text-center">
            <div>
              <div className="text-xs text-slate-400">Average</div>
              <div className={`text-lg font-mono font-bold ${
                status.latencyStats.avg_ms < 10 ? 'text-emerald-400' :
                status.latencyStats.avg_ms < 20 ? 'text-amber-400' :
                'text-rose-400'
              }`}>
                {status.latencyStats.avg_ms.toFixed(2)}ms
              </div>
            </div>

            <div>
              <div className="text-xs text-slate-400">Min</div>
              <div className="text-lg font-mono font-bold text-slate-300">
                {status.latencyStats.min_ms.toFixed(2)}ms
              </div>
            </div>

            <div>
              <div className="text-xs text-slate-400">Max</div>
              <div className="text-lg font-mono font-bold text-slate-300">
                {status.latencyStats.max_ms.toFixed(2)}ms
              </div>
            </div>
          </div>

          <div className="text-xs text-slate-500 text-center">
            {status.latencyStats.sample_count} samples
          </div>

          {/* Performance Warning */}
          {status.latencyStats.avg_ms >= 10 && (
            <div className="p-2 bg-amber-900/20 border border-amber-800 rounded">
              <p className="text-xs text-amber-400">
                ⚠️ Latency exceeds 10ms target. Check system load and device connections.
              </p>
            </div>
          )}
        </div>
      )}

      {/* No Mappings Message */}
      {status.initialized && status.mappingCount === 0 && (
        <div className="p-3 bg-slate-900 border border-slate-800 rounded">
          <p className="text-sm text-slate-400 text-center">
            Create parameter mappings to enable sync
          </p>
        </div>
      )}
    </div>
  );
}
