/**
 * Tests for MessageMonitor component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { MessageMonitor } from './MessageMonitor';

// Mock the useMessageMonitor hook
vi.mock('../hooks/useMessageMonitor', () => ({
  useMessageMonitor: () => ({
    messages: [],
    clearMessages: vi.fn(),
    isMonitoring: true,
    toggleMonitoring: vi.fn(),
    addMessage: vi.fn(),
  }),
}));

describe('MessageMonitor', () => {
  it('should render collapsed by default', () => {
    render(<MessageMonitor defaultCollapsed={true} />);
    
    expect(screen.getByText('Message Monitor')).toBeInTheDocument();
    // When collapsed, the toolbar should not be visible
    expect(screen.queryByText('Pause')).not.toBeInTheDocument();
  });

  it('should expand when header is clicked', () => {
    render(<MessageMonitor defaultCollapsed={true} />);
    
    const header = screen.getByRole('button', { expanded: false });
    fireEvent.click(header);
    
    // After expanding, toolbar should be visible
    expect(screen.getByText('Pause')).toBeInTheDocument();
    expect(screen.getByText('Clear')).toBeInTheDocument();
  });

  it('should show message count badge', () => {
    render(<MessageMonitor />);
    
    // Badge shows 0 when no messages
    expect(screen.getByText('0')).toBeInTheDocument();
  });

  it('should show recording status when monitoring', () => {
    render(<MessageMonitor />);
    
    expect(screen.getByText('Recording')).toBeInTheDocument();
  });

  it('should show empty state message when expanded with no messages', () => {
    render(<MessageMonitor defaultCollapsed={false} />);
    
    expect(screen.getByText(/No messages captured yet/)).toBeInTheDocument();
  });

  it('should render with custom maxMessages prop', () => {
    render(<MessageMonitor maxMessages={50} />);
    
    // Component should render without errors
    expect(screen.getByText('Message Monitor')).toBeInTheDocument();
  });

  it('should have accessible expand/collapse button', () => {
    render(<MessageMonitor defaultCollapsed={true} />);
    
    const button = screen.getByRole('button');
    expect(button).toHaveAttribute('aria-expanded', 'false');
    expect(button).toHaveAttribute('aria-controls', 'message-monitor-content');
  });

  it('should toggle aria-expanded when clicked', () => {
    render(<MessageMonitor defaultCollapsed={true} />);
    
    const button = screen.getByRole('button');
    expect(button).toHaveAttribute('aria-expanded', 'false');
    
    fireEvent.click(button);
    expect(button).toHaveAttribute('aria-expanded', 'true');
  });
});
