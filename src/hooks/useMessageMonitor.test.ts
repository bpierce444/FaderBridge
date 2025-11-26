/**
 * Tests for useMessageMonitor hook
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useMessageMonitor } from './useMessageMonitor';

// Mock Tauri event listener
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe('useMessageMonitor', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('should initialize with empty messages', () => {
    const { result } = renderHook(() => useMessageMonitor());
    
    expect(result.current.messages).toEqual([]);
    expect(result.current.isMonitoring).toBe(true);
  });

  it('should add messages manually', () => {
    const { result } = renderHook(() => useMessageMonitor());

    act(() => {
      result.current.addMessage({
        direction: 'incoming',
        source: 'midi',
        summary: 'CC 7 Ch1 = 127',
      });
    });

    expect(result.current.messages).toHaveLength(1);
    const msg = result.current.messages[0];
    expect(msg).toBeDefined();
    expect(msg?.summary).toBe('CC 7 Ch1 = 127');
    expect(msg?.source).toBe('midi');
    expect(msg?.direction).toBe('incoming');
  });

  it('should clear messages', () => {
    const { result } = renderHook(() => useMessageMonitor());

    act(() => {
      result.current.addMessage({
        direction: 'incoming',
        source: 'midi',
        summary: 'Test message',
      });
    });

    expect(result.current.messages).toHaveLength(1);

    act(() => {
      result.current.clearMessages();
    });

    expect(result.current.messages).toHaveLength(0);
  });

  it('should toggle monitoring', () => {
    const { result } = renderHook(() => useMessageMonitor());

    expect(result.current.isMonitoring).toBe(true);

    act(() => {
      result.current.toggleMonitoring();
    });

    expect(result.current.isMonitoring).toBe(false);

    act(() => {
      result.current.toggleMonitoring();
    });

    expect(result.current.isMonitoring).toBe(true);
  });

  it('should respect maxMessages limit', () => {
    const { result } = renderHook(() => useMessageMonitor({ maxMessages: 3 }));

    act(() => {
      for (let i = 0; i < 5; i++) {
        result.current.addMessage({
          direction: 'incoming',
          source: 'midi',
          summary: `Message ${i}`,
        });
      }
    });

    expect(result.current.messages).toHaveLength(3);
    // Newest messages should be first
    expect(result.current.messages[0]?.summary).toBe('Message 4');
    expect(result.current.messages[2]?.summary).toBe('Message 2');
  });

  it('should generate unique IDs for messages', () => {
    const { result } = renderHook(() => useMessageMonitor());

    act(() => {
      result.current.addMessage({
        direction: 'incoming',
        source: 'midi',
        summary: 'Message 1',
      });
      result.current.addMessage({
        direction: 'outgoing',
        source: 'ucnet',
        summary: 'Message 2',
      });
    });

    expect(result.current.messages[0]?.id).not.toBe(result.current.messages[1]?.id);
  });

  it('should include timestamps on messages', () => {
    const { result } = renderHook(() => useMessageMonitor());

    const beforeAdd = new Date();

    act(() => {
      result.current.addMessage({
        direction: 'incoming',
        source: 'midi',
        summary: 'Test',
      });
    });

    const afterAdd = new Date();

    const timestamp = result.current.messages[0]?.timestamp;
    expect(timestamp).toBeInstanceOf(Date);
    expect(timestamp?.getTime()).toBeGreaterThanOrEqual(beforeAdd.getTime());
    expect(timestamp?.getTime()).toBeLessThanOrEqual(afterAdd.getTime());
  });

  it('should start disabled when enabled option is false', () => {
    const { result } = renderHook(() => useMessageMonitor({ enabled: false }));

    expect(result.current.isMonitoring).toBe(false);
  });
});
