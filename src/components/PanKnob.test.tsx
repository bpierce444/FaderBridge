import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { PanKnob } from './PanKnob';

describe('PanKnob', () => {
  const defaultProps = {
    value: 0.5,
    onChange: vi.fn(),
    label: 'Pan',
  };

  it('renders with correct label', () => {
    render(<PanKnob {...defaultProps} />);
    expect(screen.getByRole('slider', { name: 'Pan' })).toBeInTheDocument();
    expect(screen.getByText('Pan')).toBeInTheDocument();
  });

  it('displays center position correctly', () => {
    render(<PanKnob {...defaultProps} value={0.5} />);
    expect(screen.getByText('C')).toBeInTheDocument();
  });

  it('displays left position correctly', () => {
    render(<PanKnob {...defaultProps} value={0.25} />);
    expect(screen.getByText('L50')).toBeInTheDocument();
  });

  it('displays right position correctly', () => {
    render(<PanKnob {...defaultProps} value={0.75} />);
    expect(screen.getByText('R50')).toBeInTheDocument();
  });

  it('displays full left position', () => {
    render(<PanKnob {...defaultProps} value={0} />);
    expect(screen.getByText('L100')).toBeInTheDocument();
  });

  it('displays full right position', () => {
    render(<PanKnob {...defaultProps} value={1} />);
    expect(screen.getByText('R100')).toBeInTheDocument();
  });

  it('has correct ARIA attributes', () => {
    render(<PanKnob {...defaultProps} value={0.75} />);
    const slider = screen.getByRole('slider');
    expect(slider).toHaveAttribute('aria-valuemin', '0');
    expect(slider).toHaveAttribute('aria-valuemax', '100');
    expect(slider).toHaveAttribute('aria-valuenow', '75');
    expect(slider).toHaveAttribute('aria-valuetext', 'R50');
  });

  it('responds to arrow key navigation', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<PanKnob {...defaultProps} value={0.5} onChange={onChange} />);

    const slider = screen.getByRole('slider');
    slider.focus();

    await user.keyboard('{ArrowRight}');
    expect(onChange).toHaveBeenCalledWith(0.55);

    await user.keyboard('{ArrowLeft}');
    expect(onChange).toHaveBeenCalledWith(0.45);
  });

  it('responds to Home key (full left)', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<PanKnob {...defaultProps} value={0.5} onChange={onChange} />);

    const slider = screen.getByRole('slider');
    slider.focus();

    await user.keyboard('{Home}');
    expect(onChange).toHaveBeenCalledWith(0);
  });

  it('responds to End key (full right)', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<PanKnob {...defaultProps} value={0.5} onChange={onChange} />);

    const slider = screen.getByRole('slider');
    slider.focus();

    await user.keyboard('{End}');
    expect(onChange).toHaveBeenCalledWith(1);
  });

  it('responds to Space key (center)', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<PanKnob {...defaultProps} value={0.25} onChange={onChange} />);

    const slider = screen.getByRole('slider');
    slider.focus();

    await user.keyboard(' ');
    expect(onChange).toHaveBeenCalledWith(0.5);
  });

  it('applies disabled state correctly', () => {
    render(<PanKnob {...defaultProps} disabled={true} />);
    const slider = screen.getByRole('slider');
    expect(slider).toHaveAttribute('aria-disabled', 'true');
    expect(slider).toHaveAttribute('tabIndex', '-1');
  });

  it('shows activity indicator when active', () => {
    const { container } = render(<PanKnob {...defaultProps} isActive={true} />);
    const activityLight = container.querySelector('[role="status"]');
    expect(activityLight).toBeInTheDocument();
  });

  it('applies active styling when isActive is true', () => {
    const { container } = render(<PanKnob {...defaultProps} isActive={true} />);
    const knob = container.querySelector('.ring-2.ring-cyan-500\\/50');
    expect(knob).toBeInTheDocument();
  });

  it('applies custom size', () => {
    const { container } = render(<PanKnob {...defaultProps} size={80} />);
    const knob = container.querySelector('div[style*="width"]');
    expect(knob).toHaveStyle({ width: '80px', height: '80px' });
  });
});
