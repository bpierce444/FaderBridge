import { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';

export interface ParameterUpdate {
  /** UCNet parameter path (e.g., "line/ch1/vol") */
  parameter: string;
  /** Normalized value (0.0 to 1.0) */
  value: number;
  /** Raw value in parameter's native units */
  rawValue?: number;
  /** Timestamp of the update */
  timestamp: number;
}

export interface UseParameterValueOptions {
  /** UCNet parameter path to monitor */
  parameter: string;
  /** Initial value (0.0 to 1.0) */
  initialValue?: number;
  /** Callback when value changes */
  onChange?: (value: number) => void;
  /** Activity timeout in milliseconds (default: 500ms) */
  activityTimeout?: number;
}

export interface UseParameterValueResult {
  /** Current normalized value (0.0 to 1.0) */
  value: number;
  /** Whether the parameter is currently receiving updates */
  isActive: boolean;
  /** Update the parameter value */
  setValue: (newValue: number) => void;
  /** Last update timestamp */
  lastUpdate: number | null;
}

/**
 * Hook to manage real-time parameter values with activity tracking.
 * Listens for parameter updates from the Tauri backend and provides
 * activity indication that fades after the specified timeout.
 */
export function useParameterValue({
  parameter,
  initialValue = 0,
  onChange,
  activityTimeout = 500,
}: UseParameterValueOptions): UseParameterValueResult {
  const [value, setValue] = useState<number>(initialValue);
  const [isActive, setIsActive] = useState<boolean>(false);
  const [lastUpdate, setLastUpdate] = useState<number | null>(null);

  // Handle incoming parameter updates from backend
  useEffect(() => {
    const unlisten = listen<ParameterUpdate>('parameter-update', (event) => {
      if (event.payload.parameter === parameter) {
        const newValue = event.payload.value;
        setValue(newValue);
        setLastUpdate(event.payload.timestamp);
        setIsActive(true);
        onChange?.(newValue);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [parameter, onChange]);

  // Auto-deactivate after timeout
  useEffect(() => {
    if (!isActive) return;

    const timer = setTimeout(() => {
      setIsActive(false);
    }, activityTimeout);

    return () => clearTimeout(timer);
  }, [isActive, lastUpdate, activityTimeout]);

  // Manual value setter (for user interaction)
  const handleSetValue = useCallback(
    (newValue: number) => {
      const clampedValue = Math.max(0, Math.min(1, newValue));
      setValue(clampedValue);
      setLastUpdate(Date.now());
      setIsActive(true);
      onChange?.(clampedValue);
    },
    [onChange]
  );

  return {
    value,
    isActive,
    setValue: handleSetValue,
    lastUpdate,
  };
}
