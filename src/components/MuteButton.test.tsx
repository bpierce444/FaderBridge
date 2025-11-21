import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { MuteButton } from './MuteButton';

describe('MuteButton', () => {
  const defaultProps = {
    muted: false,
    onToggle: vi.fn(),
  };

  it('renders with default label', () => {
    render(<MuteButton {...defaultProps} />);
    expect(screen.getByRole('switch', { name: 'Mute' })).toBeInTheDocument();
    expect(screen.getByText('Mute')).toBeInTheDocument();
  });

  it('renders with custom label', () => {
    render(<MuteButton {...defaultProps} label="Custom Mute" />);
    expect(screen.getByRole('switch', { name: 'Custom Mute' })).toBeInTheDocument();
    expect(screen.getByText('Custom Mute')).toBeInTheDocument();
  });

  it('shows correct aria-checked state when muted', () => {
    render(<MuteButton {...defaultProps} muted={true} />);
    const button = screen.getByRole('switch');
    expect(button).toHaveAttribute('aria-checked', 'true');
  });

  it('shows correct aria-checked state when unmuted', () => {
    render(<MuteButton {...defaultProps} muted={false} />);
    const button = screen.getByRole('switch');
    expect(button).toHaveAttribute('aria-checked', 'false');
  });

  it('calls onToggle when clicked', async () => {
    const user = userEvent.setup();
    const onToggle = vi.fn();
    render(<MuteButton {...defaultProps} onToggle={onToggle} />);

    const button = screen.getByRole('switch');
    await user.click(button);

    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it('calls onToggle when Enter key is pressed', async () => {
    const user = userEvent.setup();
    const onToggle = vi.fn();
    render(<MuteButton {...defaultProps} onToggle={onToggle} />);

    const button = screen.getByRole('switch');
    button.focus();
    await user.keyboard('{Enter}');

    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it('calls onToggle when Space key is pressed', async () => {
    const user = userEvent.setup();
    const onToggle = vi.fn();
    render(<MuteButton {...defaultProps} onToggle={onToggle} />);

    const button = screen.getByRole('switch');
    button.focus();
    await user.keyboard(' ');

    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it('does not call onToggle when disabled', async () => {
    const user = userEvent.setup();
    const onToggle = vi.fn();
    render(<MuteButton {...defaultProps} onToggle={onToggle} disabled={true} />);

    const button = screen.getByRole('switch');
    await user.click(button);

    expect(onToggle).not.toHaveBeenCalled();
  });

  it('applies disabled styling when disabled', () => {
    render(<MuteButton {...defaultProps} disabled={true} />);
    const button = screen.getByRole('switch');
    expect(button).toHaveAttribute('aria-disabled', 'true');
    expect(button).toHaveClass('opacity-50', 'cursor-not-allowed');
  });

  it('applies muted styling when muted', () => {
    render(<MuteButton {...defaultProps} muted={true} />);
    const button = screen.getByRole('switch');
    expect(button).toHaveClass('bg-rose-600');
  });

  it('applies unmuted styling when not muted', () => {
    render(<MuteButton {...defaultProps} muted={false} />);
    const button = screen.getByRole('switch');
    expect(button).toHaveClass('bg-slate-700');
  });

  it('shows activity indicator when active', () => {
    const { container } = render(<MuteButton {...defaultProps} isActive={true} />);
    const activityLight = container.querySelector('[role="status"]');
    expect(activityLight).toBeInTheDocument();
  });
});
