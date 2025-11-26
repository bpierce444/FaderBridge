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

vi.mock('../hooks/useActivityIndicator', () => ({
  useActivityIndicator: vi.fn(() => ({
    isActive: false,
    triggerActivity: vi.fn(),
    reset: vi.fn(),
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

  it('renders activity lights with proper aria labels', () => {
    render(<StatusBar projectId={null} showMidiActivity={true} showUCNetActivity={true} />);
    
    expect(screen.getByRole('status', { name: /midi activity/i })).toBeInTheDocument();
    expect(screen.getByRole('status', { name: /ucnet activity/i })).toBeInTheDocument();
  });

  it('shows active MIDI indicator when activity is detected', async () => {
    const useActivityIndicator = await import('../hooks/useActivityIndicator');
    vi.mocked(useActivityIndicator.useActivityIndicator).mockImplementation(({ eventNames }) => {
      // Return active for MIDI events
      const isMidi = eventNames.some(e => e.includes('midi'));
      return {
        isActive: isMidi,
        triggerActivity: vi.fn(),
        reset: vi.fn(),
      };
    });
    
    render(<StatusBar projectId={null} showMidiActivity={true} showUCNetActivity={true} />);
    
    // MIDI activity indicator should be present
    const midiIndicator = screen.getByRole('status', { name: /midi activity/i });
    expect(midiIndicator).toBeInTheDocument();
  });

  it('shows active UCNet indicator when activity is detected', async () => {
    const useActivityIndicator = await import('../hooks/useActivityIndicator');
    vi.mocked(useActivityIndicator.useActivityIndicator).mockImplementation(({ eventNames }) => {
      // Return active for UCNet events
      const isUcnet = eventNames.some(e => e.includes('ucnet') || e.includes('sync'));
      return {
        isActive: isUcnet,
        triggerActivity: vi.fn(),
        reset: vi.fn(),
      };
    });
    
    render(<StatusBar projectId={null} showMidiActivity={true} showUCNetActivity={true} />);
    
    // UCNet activity indicator should be present
    const ucnetIndicator = screen.getByRole('status', { name: /ucnet activity/i });
    expect(ucnetIndicator).toBeInTheDocument();
  });
});
