import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { ActivityLight } from './ActivityLight';

describe('ActivityLight', () => {
  it('renders when active', () => {
    render(<ActivityLight active={true} ariaLabel="Test activity" />);
    const indicator = screen.getByRole('status', { name: 'Test activity' });
    expect(indicator).toBeInTheDocument();
  });

  it('does not render when inactive', () => {
    const { container } = render(<ActivityLight active={false} ariaLabel="Test activity" />);
    // The container should be empty when inactive
    const status = container.querySelector('[role="status"]');
    expect(status).toBeInTheDocument();
    expect(status?.querySelector('div')).not.toBeInTheDocument();
  });

  it('applies correct color classes', () => {
    const { container, rerender } = render(<ActivityLight active={true} color="emerald" />);
    let light = container.querySelector('.bg-emerald-500');
    expect(light).toBeInTheDocument();

    rerender(<ActivityLight active={true} color="cyan" />);
    light = container.querySelector('.bg-cyan-500');
    expect(light).toBeInTheDocument();

    rerender(<ActivityLight active={true} color="amber" />);
    light = container.querySelector('.bg-amber-500');
    expect(light).toBeInTheDocument();
  });

  it('applies custom size', () => {
    const { container } = render(<ActivityLight active={true} size={16} />);
    const light = container.querySelector('div[style*="width"]');
    expect(light).toHaveStyle({ width: '16px', height: '16px' });
  });

  it('has proper accessibility attributes', () => {
    render(<ActivityLight active={true} ariaLabel="Volume activity" />);
    const indicator = screen.getByRole('status', { name: 'Volume activity' });
    expect(indicator).toHaveAttribute('aria-live', 'polite');
  });
});
