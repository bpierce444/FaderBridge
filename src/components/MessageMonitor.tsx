/**
 * MessageMonitor component
 * Collapsible panel at the bottom of the UI showing device message traffic
 */

import { useState, useRef, useEffect } from 'react';
import { useMessageMonitor, DeviceMessage } from '../hooks/useMessageMonitor';

export interface MessageMonitorProps {
  /** Default collapsed state */
  defaultCollapsed?: boolean;
  /** Maximum messages to display */
  maxMessages?: number;
}

/**
 * Collapsible message monitor panel for debugging device communication.
 * Shows MIDI and UCNet messages in real-time.
 * 
 * @example
 * ```tsx
 * <MessageMonitor defaultCollapsed={true} maxMessages={50} />
 * ```
 */
export function MessageMonitor({
  defaultCollapsed = true,
  maxMessages = 100,
}: MessageMonitorProps) {
  const [isCollapsed, setIsCollapsed] = useState(defaultCollapsed);
  const [autoScroll, setAutoScroll] = useState(true);
  const scrollRef = useRef<HTMLDivElement>(null);

  const {
    messages,
    clearMessages,
    isMonitoring,
    toggleMonitoring,
  } = useMessageMonitor({ maxMessages });

  // Auto-scroll to show newest messages
  useEffect(() => {
    if (autoScroll && scrollRef.current && !isCollapsed) {
      scrollRef.current.scrollTop = 0;
    }
  }, [messages, autoScroll, isCollapsed]);

  const toggleCollapsed = () => {
    setIsCollapsed((prev) => !prev);
  };

  return (
    <div className="flex flex-col bg-slate-900 border-t border-slate-700">
      {/* Header Bar - Always visible */}
      <button
        onClick={toggleCollapsed}
        className="flex items-center justify-between px-4 py-2 hover:bg-slate-800 transition-colors cursor-pointer w-full text-left"
        aria-expanded={!isCollapsed}
        aria-controls="message-monitor-content"
      >
        <div className="flex items-center gap-3">
          {/* Collapse/Expand Icon */}
          <svg
            className={`w-4 h-4 text-slate-400 transition-transform ${isCollapsed ? '' : 'rotate-180'}`}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 15l7-7 7 7" />
          </svg>
          
          <span className="text-sm font-medium text-slate-300">Message Monitor</span>
          
          {/* Message count badge */}
          <span className="px-2 py-0.5 text-xs rounded-full bg-slate-700 text-slate-400">
            {messages.length}
          </span>
          
          {/* Monitoring status indicator */}
          <div className="flex items-center gap-1.5">
            <div
              className={`w-2 h-2 rounded-full ${isMonitoring ? 'bg-emerald-500 animate-pulse' : 'bg-slate-600'}`}
            />
            <span className="text-xs text-slate-500">
              {isMonitoring ? 'Recording' : 'Paused'}
            </span>
          </div>
        </div>

        {/* Quick stats when collapsed */}
        {isCollapsed && messages.length > 0 && (
          <div className="flex items-center gap-4 text-xs text-slate-500">
            <span className="flex items-center gap-1">
              <span className="w-2 h-2 rounded-full bg-cyan-500" />
              MIDI: {messages.filter((m) => m.source === 'midi').length}
            </span>
            <span className="flex items-center gap-1">
              <span className="w-2 h-2 rounded-full bg-emerald-500" />
              UCNet: {messages.filter((m) => m.source === 'ucnet').length}
            </span>
          </div>
        )}
      </button>

      {/* Expandable Content */}
      {!isCollapsed && (
        <div id="message-monitor-content" className="flex flex-col">
          {/* Toolbar */}
          <div className="flex items-center justify-between px-4 py-2 border-t border-slate-800 bg-slate-900/50">
            <div className="flex items-center gap-2">
              {/* Toggle Monitoring */}
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  toggleMonitoring();
                }}
                className={`px-3 py-1 text-xs rounded transition-colors ${
                  isMonitoring
                    ? 'bg-rose-600/20 text-rose-400 hover:bg-rose-600/30'
                    : 'bg-emerald-600/20 text-emerald-400 hover:bg-emerald-600/30'
                }`}
              >
                {isMonitoring ? 'Pause' : 'Resume'}
              </button>

              {/* Clear Messages */}
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  clearMessages();
                }}
                className="px-3 py-1 text-xs rounded bg-slate-700 text-slate-300 hover:bg-slate-600 transition-colors"
              >
                Clear
              </button>

              {/* Auto-scroll toggle */}
              <label className="flex items-center gap-2 text-xs text-slate-400 cursor-pointer">
                <input
                  type="checkbox"
                  checked={autoScroll}
                  onChange={(e) => setAutoScroll(e.target.checked)}
                  className="w-3 h-3 rounded border-slate-600 bg-slate-700 text-cyan-500 focus:ring-cyan-500 focus:ring-offset-0"
                />
                Auto-scroll
              </label>
            </div>

            {/* Filter legend */}
            <div className="flex items-center gap-4 text-xs">
              <span className="flex items-center gap-1.5">
                <span className="w-2 h-2 rounded-full bg-cyan-500" />
                <span className="text-slate-400">MIDI</span>
              </span>
              <span className="flex items-center gap-1.5">
                <span className="w-2 h-2 rounded-full bg-emerald-500" />
                <span className="text-slate-400">UCNet</span>
              </span>
              <span className="flex items-center gap-1.5">
                <svg className="w-3 h-3 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
                </svg>
                <span className="text-slate-400">In</span>
              </span>
              <span className="flex items-center gap-1.5">
                <svg className="w-3 h-3 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 10l7-7m0 0l7 7m-7-7v18" />
                </svg>
                <span className="text-slate-400">Out</span>
              </span>
            </div>
          </div>

          {/* Message List */}
          <div
            ref={scrollRef}
            className="h-40 overflow-y-auto overflow-x-hidden border-t border-slate-800"
          >
            {messages.length === 0 ? (
              <div className="flex items-center justify-center h-full text-sm text-slate-500">
                No messages captured yet. Move a fader or interact with a device.
              </div>
            ) : (
              <div className="font-mono text-xs">
                {messages.map((message) => (
                  <MessageRow key={message.id} message={message} />
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

interface MessageRowProps {
  message: DeviceMessage;
}

/**
 * Individual message row in the monitor
 */
function MessageRow({ message }: MessageRowProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  const sourceColor = message.source === 'midi' ? 'bg-cyan-500' : 'bg-emerald-500';
  const directionIcon = message.direction === 'incoming' ? (
    <svg className="w-3 h-3 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
    </svg>
  ) : (
    <svg className="w-3 h-3 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 10l7-7m0 0l7 7m-7-7v18" />
    </svg>
  );

  const formatTime = (date: Date) => {
    const timeStr = date.toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
    const ms = date.getMilliseconds().toString().padStart(3, '0');
    return `${timeStr}.${ms}`;
  };

  return (
    <div
      className="flex items-start gap-2 px-4 py-1.5 hover:bg-slate-800/50 border-b border-slate-800/50 cursor-pointer"
      onClick={() => setIsExpanded(!isExpanded)}
    >
      {/* Timestamp */}
      <span className="text-slate-500 flex-shrink-0 w-24">
        {formatTime(message.timestamp)}
      </span>

      {/* Source indicator */}
      <span className={`w-2 h-2 rounded-full ${sourceColor} flex-shrink-0 mt-1`} />

      {/* Direction */}
      <span className="flex-shrink-0">{directionIcon}</span>

      {/* Source label */}
      <span className={`flex-shrink-0 w-12 ${message.source === 'midi' ? 'text-cyan-400' : 'text-emerald-400'}`}>
        {message.source.toUpperCase()}
      </span>

      {/* Message content */}
      <div className="flex-1 min-w-0">
        <span className="text-slate-300 break-all">{message.summary}</span>
        
        {/* Expanded raw data */}
        {isExpanded && message.rawData && (
          <div className="mt-1 p-2 bg-slate-900 rounded text-slate-500 text-[10px] break-all">
            {message.rawData}
          </div>
        )}
      </div>
    </div>
  );
}
