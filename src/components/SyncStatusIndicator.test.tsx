/**
 * Tests for SyncStatusIndicator component
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { SyncStatusIndicator } from './SyncStatusIndicator';
import * as useActiveSyncModule from '../hooks/useActiveSync';

// Mock the hook
vi.mock('../hooks/useActiveSync');

describe('SyncStatusIndicator', () => {
  const mockUseActiveSync = {
    status: {
      initialized: false,
      active: false,
      mappingCount: 0,
      latencyStats: null,
    },
    loading: false,
    error: null,
    initializeSync: vi.fn().mockResolvedValue(true),
    startSync: vi.fn().mockResolvedValue(true),
    stopSync: vi.fn().mockResolvedValue(true),
    refreshStatus: vi.fn().mockResolvedValue(undefined),
    clearLatencyStats: vi.fn().mockResolvedValue(undefined),
  };

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue(mockUseActiveSync);
  });

  it('shows loading state', () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      loading: true,
    });

    render(<SyncStatusIndicator />);

    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('shows error state', () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      error: 'Test error',
    });

    render(<SyncStatusIndicator />);

    expect(screen.getByText(/error: test error/i)).toBeInTheDocument();
  });

  it('shows not initialized state', () => {
    render(<SyncStatusIndicator />);

    expect(screen.getByText('Not Initialized')).toBeInTheDocument();
  });

  it('shows inactive state when initialized but not active', () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      status: {
        initialized: true,
        active: false,
        mappingCount: 0,
        latencyStats: null,
      },
    });

    render(<SyncStatusIndicator />);

    expect(screen.getByText('Inactive')).toBeInTheDocument();
  });

  it('shows active state with mappings', () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      status: {
        initialized: true,
        active: true,
        mappingCount: 3,
        latencyStats: null,
      },
    });

    render(<SyncStatusIndicator />);

    expect(screen.getByText('Active')).toBeInTheDocument();
    expect(screen.getByText('3 mappings')).toBeInTheDocument();
  });

  it('shows start button when inactive with mappings', () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      status: {
        initialized: true,
        active: false,
        mappingCount: 1,
        latencyStats: null,
      },
    });

    render(<SyncStatusIndicator />);

    const startButton = screen.getByRole('button', { name: /start sync/i });
    expect(startButton).toBeInTheDocument();
  });

  it('shows stop button when active', () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      status: {
        initialized: true,
        active: true,
        mappingCount: 1,
        latencyStats: null,
      },
    });

    render(<SyncStatusIndicator />);

    const stopButton = screen.getByRole('button', { name: /stop sync/i });
    expect(stopButton).toBeInTheDocument();
  });

  it('calls startSync when start button is clicked', async () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      status: {
        initialized: true,
        active: false,
        mappingCount: 1,
        latencyStats: null,
      },
    });

    render(<SyncStatusIndicator />);

    const startButton = screen.getByRole('button', { name: /start sync/i });
    fireEvent.click(startButton);

    await waitFor(() => {
      expect(mockUseActiveSync.startSync).toHaveBeenCalled();
    });
  });

  it('calls stopSync when stop button is clicked', async () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      status: {
        initialized: true,
        active: true,
        mappingCount: 1,
        latencyStats: null,
      },
    });

    render(<SyncStatusIndicator />);

    const stopButton = screen.getByRole('button', { name: /stop sync/i });
    fireEvent.click(stopButton);

    await waitFor(() => {
      expect(mockUseActiveSync.stopSync).toHaveBeenCalled();
    });
  });

  it('shows latency stats when showDetails is true', () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      status: {
        initialized: true,
        active: true,
        mappingCount: 1,
        latencyStats: {
          avg_ms: 5.5,
          min_ms: 2.0,
          max_ms: 12.0,
          sample_count: 100,
        },
      },
    });

    render(<SyncStatusIndicator showDetails={true} />);

    expect(screen.getByText('Latency Statistics')).toBeInTheDocument();
    expect(screen.getByText('5.50ms')).toBeInTheDocument();
    expect(screen.getByText('2.00ms')).toBeInTheDocument();
    expect(screen.getByText('12.00ms')).toBeInTheDocument();
    expect(screen.getByText('100 samples')).toBeInTheDocument();
  });

  it('does not show latency stats when showDetails is false', () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      status: {
        initialized: true,
        active: true,
        mappingCount: 1,
        latencyStats: {
          avg_ms: 5.5,
          min_ms: 2.0,
          max_ms: 12.0,
          sample_count: 100,
        },
      },
    });

    render(<SyncStatusIndicator showDetails={false} />);

    expect(screen.queryByText('Latency Statistics')).not.toBeInTheDocument();
  });

  it('shows performance warning when latency exceeds 10ms', () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      status: {
        initialized: true,
        active: true,
        mappingCount: 1,
        latencyStats: {
          avg_ms: 15.5,
          min_ms: 10.0,
          max_ms: 25.0,
          sample_count: 50,
        },
      },
    });

    render(<SyncStatusIndicator showDetails={true} />);

    expect(screen.getByText(/latency exceeds 10ms target/i)).toBeInTheDocument();
  });

  it('calls clearLatencyStats when clear button is clicked', async () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      status: {
        initialized: true,
        active: true,
        mappingCount: 1,
        latencyStats: {
          avg_ms: 5.5,
          min_ms: 2.0,
          max_ms: 12.0,
          sample_count: 100,
        },
      },
    });

    render(<SyncStatusIndicator showDetails={true} />);

    const clearButton = screen.getByRole('button', { name: /clear latency stats/i });
    fireEvent.click(clearButton);

    await waitFor(() => {
      expect(mockUseActiveSync.clearLatencyStats).toHaveBeenCalled();
    });
  });

  it('shows no mappings message when initialized with no mappings', () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      status: {
        initialized: true,
        active: false,
        mappingCount: 0,
        latencyStats: null,
      },
    });

    render(<SyncStatusIndicator />);

    expect(screen.getByText(/create parameter mappings to enable sync/i)).toBeInTheDocument();
  });

  it('shows correct color for good latency (< 10ms)', () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      status: {
        initialized: true,
        active: true,
        mappingCount: 1,
        latencyStats: {
          avg_ms: 5.0,
          min_ms: 2.0,
          max_ms: 8.0,
          sample_count: 100,
        },
      },
    });

    const { container } = render(<SyncStatusIndicator showDetails={true} />);

    // Check for emerald color (good latency)
    const avgLatencyElement = container.querySelector('.text-emerald-400');
    expect(avgLatencyElement).toBeInTheDocument();
  });

  it('shows correct color for slow latency (10-20ms)', () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      status: {
        initialized: true,
        active: true,
        mappingCount: 1,
        latencyStats: {
          avg_ms: 15.0,
          min_ms: 10.0,
          max_ms: 20.0,
          sample_count: 100,
        },
      },
    });

    const { container } = render(<SyncStatusIndicator showDetails={true} />);

    // Check for amber color (slow latency)
    const avgLatencyElement = container.querySelector('.text-amber-400');
    expect(avgLatencyElement).toBeInTheDocument();
  });

  it('shows correct color for lagging latency (> 20ms)', () => {
    vi.mocked(useActiveSyncModule.useActiveSync).mockReturnValue({
      ...mockUseActiveSync,
      status: {
        initialized: true,
        active: true,
        mappingCount: 1,
        latencyStats: {
          avg_ms: 25.0,
          min_ms: 15.0,
          max_ms: 35.0,
          sample_count: 100,
        },
      },
    });

    const { container } = render(<SyncStatusIndicator showDetails={true} />);

    // Check for rose color (lagging latency)
    const avgLatencyElement = container.querySelector('.text-rose-400');
    expect(avgLatencyElement).toBeInTheDocument();
  });
});
