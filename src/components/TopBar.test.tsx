/**
 * Tests for TopBar component
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { TopBar } from './TopBar';

// Mock hooks
vi.mock('../hooks/useProjects', () => ({
  useProjects: vi.fn(() => ({
    activeProject: null,
    createProject: vi.fn(),
    exportProjectToFile: vi.fn(),
    importProjectFromFile: vi.fn(),
  })),
}));

vi.mock('../hooks/useFileDialog', () => ({
  useFileDialog: vi.fn(() => ({
    saveFile: vi.fn(),
    openFile: vi.fn(),
  })),
}));

describe('TopBar', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders app title', () => {
    render(<TopBar />);
    expect(screen.getByText('FaderBridge')).toBeInTheDocument();
  });

  it('renders project management buttons', () => {
    render(<TopBar />);
    
    expect(screen.getByRole('button', { name: /new project/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /save project/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /export project/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /import project/i })).toBeInTheDocument();
  });

  it('disables save and export when no active project', () => {
    render(<TopBar />);
    
    const saveButton = screen.getByRole('button', { name: /save project/i });
    const exportButton = screen.getByRole('button', { name: /export project/i });
    
    expect(saveButton).toBeDisabled();
    expect(exportButton).toBeDisabled();
  });

  it('opens new project dialog when New is clicked', async () => {
    render(<TopBar />);
    
    const newButton = screen.getByRole('button', { name: /new project/i });
    fireEvent.click(newButton);
    
    await waitFor(() => {
      expect(screen.getByText('Create New Project')).toBeInTheDocument();
    });
  });

  it('closes new project dialog when Cancel is clicked', async () => {
    render(<TopBar />);
    
    // Open dialog
    const newButton = screen.getByRole('button', { name: /new project/i });
    fireEvent.click(newButton);
    
    await waitFor(() => {
      expect(screen.getByText('Create New Project')).toBeInTheDocument();
    });
    
    // Close dialog
    const cancelButton = screen.getByRole('button', { name: /cancel/i });
    fireEvent.click(cancelButton);
    
    await waitFor(() => {
      expect(screen.queryByText('Create New Project')).not.toBeInTheDocument();
    });
  });

  it('calls onSave when save button is clicked', async () => {
    const onSave = vi.fn();
    const useProjects = await import('../hooks/useProjects');
    vi.mocked(useProjects.useProjects).mockReturnValue({
      activeProject: { 
        id: 1, 
        name: 'Test Project', 
        description: null,
        created_at: new Date().toISOString(), 
        updated_at: new Date().toISOString(),
        last_opened_at: null,
        is_active: true,
      },
      projects: [],
      recentProjects: [],
      loading: false,
      error: null,
      createProject: vi.fn(),
      exportProjectToFile: vi.fn(),
      importProjectFromFile: vi.fn(),
      setActiveProject: vi.fn(),
      deleteProject: vi.fn(),
      updateProject: vi.fn(),
    } as any);
    
    render(<TopBar onSave={onSave} />);
    
    const saveButton = screen.getByRole('button', { name: /save project/i });
    fireEvent.click(saveButton);
    
    expect(onSave).toHaveBeenCalled();
  });

  it('displays active project name', async () => {
    const useProjects = await import('../hooks/useProjects');
    vi.mocked(useProjects.useProjects).mockReturnValue({
      activeProject: { 
        id: 1, 
        name: 'My Studio Setup', 
        description: null,
        created_at: new Date().toISOString(), 
        updated_at: new Date().toISOString(),
        last_opened_at: null,
        is_active: true,
      },
      projects: [],
      recentProjects: [],
      loading: false,
      error: null,
      createProject: vi.fn(),
      exportProjectToFile: vi.fn(),
      importProjectFromFile: vi.fn(),
      setActiveProject: vi.fn(),
      deleteProject: vi.fn(),
      updateProject: vi.fn(),
    } as any);
    
    render(<TopBar />);
    expect(screen.getByText('My Studio Setup')).toBeInTheDocument();
  });
});
