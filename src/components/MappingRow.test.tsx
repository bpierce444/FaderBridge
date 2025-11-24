/**
 * Tests for MappingRow component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { MappingRow } from './MappingRow';
import type { Mapping, Device } from '../hooks/useMappings';

describe('MappingRow', () => {
  const mockMapping: Mapping = {
    id: 1,
    project_id: 1,
    midi_device_id: 1,
    ucnet_device_id: 2,
    midi_channel: 0,
    midi_cc: 7,
    ucnet_parameter_id: 100,
    ucnet_parameter_name: 'Channel 1 Volume',
    taper_curve: 'linear',
    min_value: 0,
    max_value: 1,
    invert: false,
    bidirectional: false,
    label: null,
    created_at: '2024-01-01T00:00:00Z',
  };

  const mockDevices: Device[] = [
    {
      id: 1,
      project_id: 1,
      device_type: 'midi',
      device_name: 'Test MIDI Controller',
      device_id: 'midi-1',
      connection_type: null,
      config_json: null,
      created_at: '2024-01-01T00:00:00Z',
    },
    {
      id: 2,
      project_id: 1,
      device_type: 'ucnet',
      device_name: 'Test UCNet Device',
      device_id: 'ucnet-1',
      connection_type: null,
      config_json: null,
      created_at: '2024-01-01T00:00:00Z',
    },
  ];

  const mockOnUpdate = vi.fn().mockResolvedValue(undefined);
  const mockOnDelete = vi.fn().mockResolvedValue(undefined);

  it('renders mapping information correctly', () => {
    render(
      <MappingRow
        mapping={mockMapping}
        devices={mockDevices}
        onUpdate={mockOnUpdate}
        onDelete={mockOnDelete}
      />
    );

    expect(screen.getByText('Test MIDI Controller')).toBeInTheDocument();
    expect(screen.getByText('CH1 CC7')).toBeInTheDocument();
    expect(screen.getByText('Test UCNet Device')).toBeInTheDocument();
    expect(screen.getByText('Channel 1 Volume')).toBeInTheDocument();
    expect(screen.getByText('Curve: linear')).toBeInTheDocument();
  });

  it('enters edit mode when Edit button is clicked', async () => {
    render(
      <MappingRow
        mapping={mockMapping}
        devices={mockDevices}
        onUpdate={mockOnUpdate}
        onDelete={mockOnDelete}
      />
    );

    const editButton = screen.getByRole('button', { name: /edit mapping/i });
    fireEvent.click(editButton);

    await waitFor(() => {
      expect(screen.getByText(/editing mapping #1/i)).toBeInTheDocument();
    });

    expect(screen.getByRole('button', { name: /save changes/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /cancel editing/i })).toBeInTheDocument();
  });

  it('cancels edit mode when Cancel button is clicked', async () => {
    render(
      <MappingRow
        mapping={mockMapping}
        devices={mockDevices}
        onUpdate={mockOnUpdate}
        onDelete={mockOnDelete}
      />
    );

    // Enter edit mode
    const editButton = screen.getByRole('button', { name: /edit mapping/i });
    fireEvent.click(editButton);

    await waitFor(() => {
      expect(screen.getByText(/editing mapping #1/i)).toBeInTheDocument();
    });

    // Cancel edit
    const cancelButton = screen.getByRole('button', { name: /cancel editing/i });
    fireEvent.click(cancelButton);

    await waitFor(() => {
      expect(screen.queryByText(/editing mapping #1/i)).not.toBeInTheDocument();
    });
  });

  it('calls onUpdate when Save button is clicked', async () => {
    render(
      <MappingRow
        mapping={mockMapping}
        devices={mockDevices}
        onUpdate={mockOnUpdate}
        onDelete={mockOnDelete}
      />
    );

    // Enter edit mode
    const editButton = screen.getByRole('button', { name: /edit mapping/i });
    fireEvent.click(editButton);

    await waitFor(() => {
      expect(screen.getByText(/editing mapping #1/i)).toBeInTheDocument();
    });

    // Change taper curve
    const taperSelect = screen.getByLabelText(/taper curve/i);
    fireEvent.change(taperSelect, { target: { value: 'logarithmic' } });

    // Save changes
    const saveButton = screen.getByRole('button', { name: /save changes/i });
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(mockOnUpdate).toHaveBeenCalledWith(1, expect.objectContaining({
        taper_curve: 'logarithmic',
      }));
    });
  });

  it('calls onDelete when Delete button is clicked and confirmed', async () => {
    // Mock window.confirm
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);

    render(
      <MappingRow
        mapping={mockMapping}
        devices={mockDevices}
        onUpdate={mockOnUpdate}
        onDelete={mockOnDelete}
      />
    );

    const deleteButton = screen.getByRole('button', { name: /delete mapping/i });
    fireEvent.click(deleteButton);

    await waitFor(() => {
      expect(confirmSpy).toHaveBeenCalled();
      expect(mockOnDelete).toHaveBeenCalledWith(1);
    });

    confirmSpy.mockRestore();
  });

  it('does not call onDelete when deletion is cancelled', async () => {
    // Mock window.confirm to return false
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false);

    render(
      <MappingRow
        mapping={mockMapping}
        devices={mockDevices}
        onUpdate={mockOnUpdate}
        onDelete={mockOnDelete}
      />
    );

    const deleteButton = screen.getByRole('button', { name: /delete mapping/i });
    fireEvent.click(deleteButton);

    await waitFor(() => {
      expect(confirmSpy).toHaveBeenCalled();
      expect(mockOnDelete).not.toHaveBeenCalled();
    });

    confirmSpy.mockRestore();
  });

  it('displays label when provided', () => {
    const mappingWithLabel = { ...mockMapping, label: 'Main Fader' };

    render(
      <MappingRow
        mapping={mappingWithLabel}
        devices={mockDevices}
        onUpdate={mockOnUpdate}
        onDelete={mockOnDelete}
      />
    );

    expect(screen.getByText('"Main Fader"')).toBeInTheDocument();
  });

  it('displays inverted indicator when mapping is inverted', () => {
    const invertedMapping = { ...mockMapping, invert: true };

    render(
      <MappingRow
        mapping={invertedMapping}
        devices={mockDevices}
        onUpdate={mockOnUpdate}
        onDelete={mockOnDelete}
      />
    );

    expect(screen.getByText('Inverted')).toBeInTheDocument();
  });

  it('displays bidirectional indicator when mapping is bidirectional', () => {
    const bidirectionalMapping = { ...mockMapping, bidirectional: true };

    render(
      <MappingRow
        mapping={bidirectionalMapping}
        devices={mockDevices}
        onUpdate={mockOnUpdate}
        onDelete={mockOnDelete}
      />
    );

    expect(screen.getByText('Bidirectional')).toBeInTheDocument();
  });

  it('allows editing all mapping properties', async () => {
    render(
      <MappingRow
        mapping={mockMapping}
        devices={mockDevices}
        onUpdate={mockOnUpdate}
        onDelete={mockOnDelete}
      />
    );

    // Enter edit mode
    const editButton = screen.getByRole('button', { name: /edit mapping/i });
    fireEvent.click(editButton);

    await waitFor(() => {
      expect(screen.getByText(/editing mapping #1/i)).toBeInTheDocument();
    });

    // Change multiple properties
    const taperSelect = screen.getByLabelText(/taper curve/i);
    fireEvent.change(taperSelect, { target: { value: 'exponential' } });

    const minValueInput = screen.getByLabelText(/min value/i);
    fireEvent.change(minValueInput, { target: { value: '0.1' } });

    const maxValueInput = screen.getByLabelText(/max value/i);
    fireEvent.change(maxValueInput, { target: { value: '0.9' } });

    const labelInput = screen.getByLabelText(/label/i);
    fireEvent.change(labelInput, { target: { value: 'Custom Label' } });

    const invertCheckbox = screen.getByLabelText(/invert/i);
    fireEvent.click(invertCheckbox);

    // Save changes
    const saveButton = screen.getByRole('button', { name: /save changes/i });
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(mockOnUpdate).toHaveBeenCalledWith(1, expect.objectContaining({
        taper_curve: 'exponential',
        min_value: 0.1,
        max_value: 0.9,
        label: 'Custom Label',
        invert: true,
      }));
    });
  });
});
