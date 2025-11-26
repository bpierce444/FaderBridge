/**
 * Hook for monitoring MIDI and UCNet messages
 * Captures device messages for debugging and visualization
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

/** Direction of the message */
export type MessageDirection = 'incoming' | 'outgoing';

/** Source of the message */
export type MessageSource = 'midi' | 'ucnet';

/** A captured device message */
export interface DeviceMessage {
  /** Unique ID for React keys */
  id: string;
  /** Timestamp when message was captured */
  timestamp: Date;
  /** Direction of the message */
  direction: MessageDirection;
  /** Source protocol */
  source: MessageSource;
  /** Human-readable message summary */
  summary: string;
  /** Raw message data (for detailed view) */
  rawData?: string;
}

export interface UseMessageMonitorOptions {
  /** Maximum number of messages to retain (default: 100) */
  maxMessages?: number;
  /** Whether monitoring is enabled (default: true) */
  enabled?: boolean;
}

export interface UseMessageMonitorResult {
  /** Array of captured messages (newest first) */
  messages: DeviceMessage[];
  /** Clear all messages */
  clearMessages: () => void;
  /** Whether monitoring is currently active */
  isMonitoring: boolean;
  /** Toggle monitoring on/off */
  toggleMonitoring: () => void;
  /** Manually add a message (useful for testing) */
  addMessage: (message: Omit<DeviceMessage, 'id' | 'timestamp'>) => void;
}

/** Event names to listen for MIDI messages */
const MIDI_EVENTS = [
  'midi:message-received',
  'midi:message-sent',
  'parameter-update',
  'sync:midi-to-ucnet',
];

/** Event names to listen for UCNet messages */
const UCNET_EVENTS = [
  'ucnet:parameter-changed',
  'ucnet:message-sent',
  'sync:ucnet-to-midi',
  'sync:parameter-synced',
];

let messageIdCounter = 0;

/**
 * Hook to monitor device messages from MIDI and UCNet.
 * Captures incoming and outgoing messages for debugging visualization.
 * 
 * @example
 * ```tsx
 * const { messages, clearMessages, isMonitoring, toggleMonitoring } = useMessageMonitor({
 *   maxMessages: 50,
 * });
 * ```
 */
export function useMessageMonitor({
  maxMessages = 100,
  enabled = true,
}: UseMessageMonitorOptions = {}): UseMessageMonitorResult {
  const [messages, setMessages] = useState<DeviceMessage[]>([]);
  const [isMonitoring, setIsMonitoring] = useState(enabled);
  const maxMessagesRef = useRef(maxMessages);

  // Keep ref in sync
  maxMessagesRef.current = maxMessages;

  /**
   * Generate a unique message ID
   */
  const generateId = useCallback(() => {
    messageIdCounter += 1;
    return `msg-${Date.now()}-${messageIdCounter}`;
  }, []);

  /**
   * Add a new message to the list
   */
  const addMessage = useCallback(
    (message: Omit<DeviceMessage, 'id' | 'timestamp'>) => {
      const newMessage: DeviceMessage = {
        ...message,
        id: generateId(),
        timestamp: new Date(),
      };

      setMessages((prev) => {
        const updated = [newMessage, ...prev];
        // Trim to max messages
        if (updated.length > maxMessagesRef.current) {
          return updated.slice(0, maxMessagesRef.current);
        }
        return updated;
      });
    },
    [generateId]
  );

  /**
   * Clear all messages
   */
  const clearMessages = useCallback(() => {
    setMessages([]);
  }, []);

  /**
   * Toggle monitoring on/off
   */
  const toggleMonitoring = useCallback(() => {
    setIsMonitoring((prev) => !prev);
  }, []);

  /**
   * Parse MIDI event payload into a message
   */
  const parseMidiEvent = useCallback(
    (eventName: string, payload: unknown): Omit<DeviceMessage, 'id' | 'timestamp'> | null => {
      const direction: MessageDirection = eventName.includes('sent') || eventName.includes('midi-to-ucnet')
        ? 'outgoing'
        : 'incoming';

      // Type guard for payload
      const data = payload as Record<string, unknown> | undefined;

      if (eventName === 'midi:message-received' || eventName === 'parameter-update') {
        const channel = data?.channel ?? '?';
        const controller = data?.controller ?? data?.cc ?? '?';
        const value = data?.value ?? '?';
        return {
          direction: 'incoming',
          source: 'midi',
          summary: `CC ${controller} Ch${channel} = ${value}`,
          rawData: JSON.stringify(data),
        };
      }

      if (eventName === 'midi:message-sent') {
        const channel = data?.channel ?? '?';
        const controller = data?.controller ?? data?.cc ?? '?';
        const value = data?.value ?? '?';
        return {
          direction: 'outgoing',
          source: 'midi',
          summary: `CC ${controller} Ch${channel} = ${value}`,
          rawData: JSON.stringify(data),
        };
      }

      if (eventName === 'sync:midi-to-ucnet') {
        const param = data?.parameter ?? data?.target ?? 'unknown';
        const value = data?.value ?? '?';
        return {
          direction: 'outgoing',
          source: 'midi',
          summary: `Sync → ${param} = ${value}`,
          rawData: JSON.stringify(data),
        };
      }

      return {
        direction,
        source: 'midi',
        summary: `MIDI: ${eventName}`,
        rawData: JSON.stringify(data),
      };
    },
    []
  );

  /**
   * Parse UCNet event payload into a message
   */
  const parseUcnetEvent = useCallback(
    (eventName: string, payload: unknown): Omit<DeviceMessage, 'id' | 'timestamp'> | null => {
      const data = payload as Record<string, unknown> | undefined;

      if (eventName === 'ucnet:parameter-changed') {
        const param = data?.parameter ?? data?.path ?? 'unknown';
        const value = data?.value ?? '?';
        return {
          direction: 'incoming',
          source: 'ucnet',
          summary: `${param} = ${value}`,
          rawData: JSON.stringify(data),
        };
      }

      if (eventName === 'ucnet:message-sent') {
        const param = data?.parameter ?? data?.path ?? 'unknown';
        const value = data?.value ?? '?';
        return {
          direction: 'outgoing',
          source: 'ucnet',
          summary: `→ ${param} = ${value}`,
          rawData: JSON.stringify(data),
        };
      }

      if (eventName === 'sync:ucnet-to-midi' || eventName === 'sync:parameter-synced') {
        const param = data?.parameter ?? data?.source ?? 'unknown';
        const value = data?.value ?? '?';
        return {
          direction: 'incoming',
          source: 'ucnet',
          summary: `Sync ← ${param} = ${value}`,
          rawData: JSON.stringify(data),
        };
      }

      return {
        direction: 'incoming',
        source: 'ucnet',
        summary: `UCNet: ${eventName}`,
        rawData: JSON.stringify(data),
      };
    },
    []
  );

  /**
   * Listen for events from the backend
   */
  useEffect(() => {
    if (!isMonitoring) return;

    const unlistenFns: Promise<UnlistenFn>[] = [];

    // Listen for MIDI events
    for (const eventName of MIDI_EVENTS) {
      const unlisten = listen(eventName, (event) => {
        const message = parseMidiEvent(eventName, event.payload);
        if (message) {
          addMessage(message);
        }
      });
      unlistenFns.push(unlisten);
    }

    // Listen for UCNet events
    for (const eventName of UCNET_EVENTS) {
      const unlisten = listen(eventName, (event) => {
        const message = parseUcnetEvent(eventName, event.payload);
        if (message) {
          addMessage(message);
        }
      });
      unlistenFns.push(unlisten);
    }

    return () => {
      for (const unlisten of unlistenFns) {
        unlisten.then((fn) => fn());
      }
    };
  }, [isMonitoring, addMessage, parseMidiEvent, parseUcnetEvent]);

  return {
    messages,
    clearMessages,
    isMonitoring,
    toggleMonitoring,
    addMessage,
  };
}
