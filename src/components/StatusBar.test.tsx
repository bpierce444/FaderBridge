/**
 * Tests for StatusBar component
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { StatusBar } from './StatusBar';

// Mock hooks
vi.mock('../hooks/useAutoSave', () => ({
  useAutoSave: vi.fn(() => ({
    isSaving: false,
    lastSaved: null,
    saveNow: vi.fn(),
    markDirty: vi.fn(),
  })),
}));

vi.mock('../hooks/useActiveSync', () => ({
  useActiveSync: vi.fn(() => ({
    status: {
      initialized: true,
      active: false,
      mappingCount: 0,
      latencyStats: null,
    },
    loading: false,
    error: null,
    startSync: vi.fn(),
    stopSync: vi.fn(),
    clearLatencyStats: vi.fn(),
  })),
}));

describe('StatusBar', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders sync status indicator', () => {
    render(<StatusBar projectId={null} />);
    // SyncStatusIndicator should be rendered
    expect(screen.getByText(/inactive/i)).toBeInTheDocument();
  });

  it('renders activity indicators when enabled', () => {
    render(<StatusBar projectId={null} showMidiActivity={true} showUCNetActivity={true} />);
    
    expect(screen.getByText('MIDI')).toBeInTheDocument();
    expect(screen.getByText('UCNet')).toBeInTheDocument();
  });

  it('hides activity indicators when disabled', () => {
    render(<StatusBar projectId={null} showMidiActivity={false} showUCNetActivity={false} />);
    
    expect(screen.queryByText('MIDI')).not.toBeInTheDocument();
    expect(screen.queryByText('UCNet')).not.toBeInTheDocument();
  });

  it('shows saving status when saving', async () => {
    const useAutoSave = await import('../hooks/useAutoSave');
    vi.mocked(useAutoSave.useAutoSave).mockReturnValue({
      isSaving: true,
      lastSaved: null,
      saveNow: vi.fn(),
      markDirty: vi.fn(),
    });
    
    render(<StatusBar projectId={1} />);
    expect(screen.getByText('Saving...')).toBeInTheDocument();
  });

  it('shows last saved time when available', async () => {
    const useAutoSave = await import('../hooks/useAutoSave');
    const lastSaved = new Date(Date.now() - 30000); // 30 seconds ago
    vi.mocked(useAutoSave.useAutoSave).mockReturnValue({
      isSaving: false,
      lastSaved,
      saveNow: vi.fn(),
      markDirty: vi.fn(),
    });
    
    render(<StatusBar projectId={1} />);
    expect(screen.getByText(/saved/i)).toBeInTheDocument();
  });

  it('displays version info', () => {
    render(<StatusBar projectId={null} />);
    expect(screen.getByText('Phase 1 MVP')).toBeInTheDocument();
  });

  it('does not show save status when no project', () => {
    render(<StatusBar projectId={null} />);
    expect(screen.queryByText(/saving/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/saved/i)).not.toBeInTheDocument();
  });
});
