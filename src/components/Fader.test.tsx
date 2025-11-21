import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { Fader } from './Fader';

describe('Fader', () => {
  const defaultProps = {
    value: 0.75,
    onChange: vi.fn(),
    label: 'Volume',
  };

  it('renders with correct label', () => {
    render(<Fader {...defaultProps} />);
    expect(screen.getByRole('slider', { name: 'Volume' })).toBeInTheDocument();
    expect(screen.getByText('Volume')).toBeInTheDocument();
  });

  it('displays dB value correctly', () => {
    render(<Fader {...defaultProps} value={0.75} showDb={true} minDb={-60} maxDb={10} />);
    // 0.75 * 70 - 60 = -7.5 dB
    expect(screen.getByText('-7.5')).toBeInTheDocument();
  });

  it('displays percentage when showDb is false', () => {
    render(<Fader {...defaultProps} value={0.75} showDb={false} />);
    expect(screen.getByText('75%')).toBeInTheDocument();
  });

  it('displays -∞ for zero value', () => {
    render(<Fader {...defaultProps} value={0} showDb={true} />);
    expect(screen.getByText('-∞')).toBeInTheDocument();
  });

  it('has correct ARIA attributes', () => {
    render(<Fader {...defaultProps} value={0.75} />);
    const slider = screen.getByRole('slider');
    expect(slider).toHaveAttribute('aria-valuemin', '0');
    expect(slider).toHaveAttribute('aria-valuemax', '100');
    expect(slider).toHaveAttribute('aria-valuenow', '75');
  });

  it('responds to keyboard navigation', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<Fader {...defaultProps} value={0.5} onChange={onChange} />);

    const slider = screen.getByRole('slider');
    slider.focus();

    await user.keyboard('{ArrowUp}');
    expect(onChange).toHaveBeenCalledWith(0.55);

    await user.keyboard('{ArrowDown}');
    expect(onChange).toHaveBeenCalledWith(0.45);
  });

  it('responds to Home and End keys', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<Fader {...defaultProps} value={0.5} onChange={onChange} />);

    const slider = screen.getByRole('slider');
    slider.focus();

    await user.keyboard('{Home}');
    expect(onChange).toHaveBeenCalledWith(1);

    await user.keyboard('{End}');
    expect(onChange).toHaveBeenCalledWith(0);
  });

  it('shows activity indicator when active', () => {
    const { container } = render(<Fader {...defaultProps} isActive={true} />);
    const activityLight = container.querySelector('[role="status"]');
    expect(activityLight).toBeInTheDocument();
  });

  it('applies disabled state correctly', () => {
    render(<Fader {...defaultProps} disabled={true} />);
    const slider = screen.getByRole('slider');
    expect(slider).toHaveAttribute('aria-disabled', 'true');
    expect(slider).toHaveAttribute('tabIndex', '-1');
  });

  it('applies active styling when isActive is true', () => {
    const { container } = render(<Fader {...defaultProps} isActive={true} />);
    const track = container.querySelector('.ring-2.ring-emerald-500\\/50');
    expect(track).toBeInTheDocument();
  });
});
