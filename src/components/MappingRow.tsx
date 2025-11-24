/**
 * MappingRow component
 * Displays a single parameter mapping with edit and delete controls
 */

import { useState } from 'react';
import { Mapping, Device, TaperCurve, UpdateMappingRequest } from '../hooks/useMappings';

export interface MappingRowProps {
  /** The mapping to display */
  mapping: Mapping;
  /** All devices for display names */
  devices: Device[];
  /** Callback when mapping is updated */
  onUpdate: (id: number, updates: Partial<UpdateMappingRequest>) => Promise<void>;
  /** Callback when mapping is deleted */
  onDelete: (id: number) => Promise<void>;
  /** Whether the row is in edit mode */
  isEditing?: boolean;
  /** Callback when edit mode changes */
  onEditChange?: (editing: boolean) => void;
}

/**
 * Component for displaying and editing a single mapping
 * 
 * @example
 * ```tsx
 * <MappingRow
 *   mapping={mapping}
 *   devices={devices}
 *   onUpdate={handleUpdate}
 *   onDelete={handleDelete}
 * />
 * ```
 */
export function MappingRow({
  mapping,
  devices,
  onUpdate,
  onDelete,
  isEditing: externalIsEditing,
  onEditChange,
}: MappingRowProps) {
  const [internalIsEditing, setInternalIsEditing] = useState(false);
  const [editValues, setEditValues] = useState({
    taper_curve: mapping.taper_curve,
    min_value: mapping.min_value,
    max_value: mapping.max_value,
    invert: mapping.invert,
    bidirectional: mapping.bidirectional,
    label: mapping.label || '',
  });

  const isEditing = externalIsEditing ?? internalIsEditing;
  const setIsEditing = onEditChange || setInternalIsEditing;

  // Get device names
  const midiDevice = devices.find(d => d.id === mapping.midi_device_id);
  const ucnetDevice = devices.find(d => d.id === mapping.ucnet_device_id);

  const handleEdit = () => {
    setIsEditing(true);
  };

  const handleCancel = () => {
    setEditValues({
      taper_curve: mapping.taper_curve,
      min_value: mapping.min_value,
      max_value: mapping.max_value,
      invert: mapping.invert,
      bidirectional: mapping.bidirectional,
      label: mapping.label || '',
    });
    setIsEditing(false);
  };

  const handleSave = async () => {
    await onUpdate(mapping.id, {
      taper_curve: editValues.taper_curve,
      min_value: editValues.min_value,
      max_value: editValues.max_value,
      invert: editValues.invert,
      bidirectional: editValues.bidirectional,
      label: editValues.label || undefined,
    });
    setIsEditing(false);
  };

  const handleDelete = async () => {
    if (window.confirm('Are you sure you want to delete this mapping?')) {
      await onDelete(mapping.id);
    }
  };

  if (isEditing) {
    return (
      <div className="p-4 bg-slate-800 border border-cyan-500 rounded-lg space-y-3">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="text-sm font-mono text-slate-400">
            Editing Mapping #{mapping.id}
          </div>
          <div className="flex gap-2">
            <button
              onClick={handleCancel}
              className="px-3 py-1 text-sm bg-slate-700 hover:bg-slate-600 text-white rounded transition-colors"
              aria-label="Cancel editing"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              className="px-3 py-1 text-sm bg-cyan-500 hover:bg-cyan-600 text-slate-950 font-semibold rounded transition-colors"
              aria-label="Save changes"
            >
              Save
            </button>
          </div>
        </div>

        {/* Edit Form */}
        <div className="grid grid-cols-2 gap-4">
          {/* Taper Curve */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-1">
              Taper Curve
            </label>
            <select
              value={editValues.taper_curve}
              onChange={(e) => setEditValues({ ...editValues, taper_curve: e.target.value as TaperCurve })}
              className="w-full px-3 py-2 bg-slate-900 border border-slate-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-cyan-500"
            >
              <option value="linear">Linear</option>
              <option value="logarithmic">Logarithmic</option>
              <option value="exponential">Exponential</option>
              <option value="s-curve">S-Curve</option>
            </select>
          </div>

          {/* Min Value */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-1">
              Min Value
            </label>
            <input
              type="number"
              value={editValues.min_value}
              onChange={(e) => setEditValues({ ...editValues, min_value: parseFloat(e.target.value) })}
              step="0.01"
              min="0"
              max="1"
              className="w-full px-3 py-2 bg-slate-900 border border-slate-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-cyan-500"
            />
          </div>

          {/* Max Value */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-1">
              Max Value
            </label>
            <input
              type="number"
              value={editValues.max_value}
              onChange={(e) => setEditValues({ ...editValues, max_value: parseFloat(e.target.value) })}
              step="0.01"
              min="0"
              max="1"
              className="w-full px-3 py-2 bg-slate-900 border border-slate-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-cyan-500"
            />
          </div>

          {/* Label */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-1">
              Label (Optional)
            </label>
            <input
              type="text"
              value={editValues.label}
              onChange={(e) => setEditValues({ ...editValues, label: e.target.value })}
              placeholder="Custom label..."
              className="w-full px-3 py-2 bg-slate-900 border border-slate-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-cyan-500"
            />
          </div>
        </div>

        {/* Checkboxes */}
        <div className="flex gap-6">
          <label className="flex items-center gap-2 text-sm text-slate-300 cursor-pointer">
            <input
              type="checkbox"
              checked={editValues.invert}
              onChange={(e) => setEditValues({ ...editValues, invert: e.target.checked })}
              className="w-4 h-4 bg-slate-900 border-slate-700 rounded text-cyan-500 focus:ring-2 focus:ring-cyan-500"
            />
            Invert
          </label>
          <label className="flex items-center gap-2 text-sm text-slate-300 cursor-pointer">
            <input
              type="checkbox"
              checked={editValues.bidirectional}
              onChange={(e) => setEditValues({ ...editValues, bidirectional: e.target.checked })}
              className="w-4 h-4 bg-slate-900 border-slate-700 rounded text-cyan-500 focus:ring-2 focus:ring-cyan-500"
            />
            Bidirectional
          </label>
        </div>
      </div>
    );
  }

  return (
    <div className="p-4 bg-slate-900 border border-slate-800 rounded-lg hover:border-slate-700 transition-colors">
      <div className="flex items-center justify-between">
        {/* Mapping Info */}
        <div className="flex-1 min-w-0 space-y-2">
          {/* Source → Target */}
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2">
              <span className="text-white font-semibold">
                {midiDevice?.device_name || 'Unknown MIDI Device'}
              </span>
              <span className="font-mono text-sm text-cyan-400">
                CH{mapping.midi_channel + 1} CC{mapping.midi_cc}
              </span>
            </div>
            
            <svg
              className="w-5 h-5 text-slate-500 flex-shrink-0"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M13 7l5 5m0 0l-5 5m5-5H6"
              />
            </svg>
            
            <div className="flex items-center gap-2">
              <span className="text-white font-semibold">
                {ucnetDevice?.device_name || 'Unknown UCNet Device'}
              </span>
              <span className="text-sm text-slate-400">
                {mapping.ucnet_parameter_name}
              </span>
            </div>
          </div>

          {/* Details */}
          <div className="flex items-center gap-4 text-sm text-slate-400">
            <span>Curve: {mapping.taper_curve}</span>
            <span>Range: {mapping.min_value.toFixed(2)} - {mapping.max_value.toFixed(2)}</span>
            {mapping.invert && <span className="text-amber-500">Inverted</span>}
            {mapping.bidirectional && <span className="text-cyan-500">Bidirectional</span>}
            {mapping.label && <span className="text-slate-300">"{mapping.label}"</span>}
          </div>
        </div>

        {/* Actions */}
        <div className="flex gap-2 ml-4">
          <button
            onClick={handleEdit}
            className="px-3 py-2 text-sm bg-slate-800 hover:bg-slate-700 text-white rounded transition-colors"
            aria-label="Edit mapping"
          >
            Edit
          </button>
          <button
            onClick={handleDelete}
            className="px-3 py-2 text-sm bg-rose-900/20 hover:bg-rose-900/40 text-rose-400 rounded transition-colors"
            aria-label="Delete mapping"
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  );
}
