/**
 * MappingManager component
 * Central UI for creating, viewing, editing, and deleting parameter mappings
 */

import { useState, useMemo } from 'react';
import { useMappings, CreateMappingRequest, UpdateMappingRequest, Mapping } from '../hooks/useMappings';
import { MappingRow } from '../components/MappingRow';
import { ParameterSelector } from '../components/ParameterSelector';
import { useMidiLearn } from '../hooks/useMidiLearn';
import { MixerStrip } from '../components/MixerStrip';

/**
 * Extracts unique UCNet channels from mappings for visual feedback display.
 * Groups by device and channel prefix (e.g., "line/ch1").
 */
function extractUniqueChannels(mappings: Mapping[]): Array<{
  key: string;
  channelNumber: number;
  label: string;
  parameterPrefix: string;
}> {
  const channelMap = new Map<string, {
    channelNumber: number;
    label: string;
    parameterPrefix: string;
  }>();

  for (const mapping of mappings) {
    // Extract channel prefix from parameter name (e.g., "line/ch1/vol" -> "line/ch1")
    const paramName = mapping.ucnet_parameter_name;
    const parts = paramName.split('/');
    
    // Assume format like "line/ch1/vol" or "ch1/vol"
    let prefix: string;
    let channelNum = 1;
    
    if (parts.length >= 2) {
      // Find the channel part (e.g., "ch1", "ch2")
      const channelPart = parts.find(p => /^ch\d+$/i.test(p));
      if (channelPart) {
        channelNum = parseInt(channelPart.replace(/\D/g, ''), 10) || 1;
        // Build prefix up to and including channel
        const channelIndex = parts.indexOf(channelPart);
        prefix = parts.slice(0, channelIndex + 1).join('/');
      } else {
        // No channel found, use first two parts
        prefix = parts.slice(0, 2).join('/');
      }
    } else {
      prefix = paramName;
    }

    const key = `${mapping.ucnet_device_id}:${prefix}`;
    
    if (!channelMap.has(key)) {
      channelMap.set(key, {
        channelNumber: channelNum,
        label: mapping.label || `Ch ${channelNum}`,
        parameterPrefix: prefix,
      });
    }
  }

  return Array.from(channelMap.entries()).map(([key, value]) => ({
    key,
    ...value,
  }));
}

export interface MappingManagerProps {
  /** Current project ID */
  projectId: number | null;
}

/**
 * Main mapping interface component
 * Provides UI for managing parameter mappings between MIDI and UCNet devices
 * 
 * @example
 * ```tsx
 * <MappingManager projectId={currentProject?.id ?? null} />
 * ```
 */
export function MappingManager({ projectId }: MappingManagerProps) {
  const { mappings, devices, loading, error, createMapping, updateMapping, deleteMapping, refresh } =
    useMappings(projectId);
  const { isLearning, startLearn, cancelLearn } = useMidiLearn();

  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newMapping, setNewMapping] = useState<Partial<CreateMappingRequest>>({
    midi_channel: 0,
    midi_cc: 7,
    taper_curve: 'linear',
    min_value: 0,
    max_value: 1,
    invert: false,
    bidirectional: false,
  });

  // Filter devices by type
  const midiDevices = devices.filter(d => d.device_type === 'midi');
  const ucnetDevices = devices.filter(d => d.device_type === 'ucnet');

  const handleCreateMapping = async () => {
    if (!projectId) return;
    if (!newMapping.midi_device_id || !newMapping.ucnet_device_id || !newMapping.ucnet_parameter_id || !newMapping.ucnet_parameter_name) {
      alert('Please fill in all required fields');
      return;
    }

    const request: CreateMappingRequest = {
      project_id: projectId,
      midi_device_id: newMapping.midi_device_id,
      ucnet_device_id: newMapping.ucnet_device_id,
      midi_channel: newMapping.midi_channel ?? 0,
      midi_cc: newMapping.midi_cc ?? 7,
      ucnet_parameter_id: newMapping.ucnet_parameter_id,
      ucnet_parameter_name: newMapping.ucnet_parameter_name,
      taper_curve: newMapping.taper_curve,
      min_value: newMapping.min_value,
      max_value: newMapping.max_value,
      invert: newMapping.invert,
      bidirectional: newMapping.bidirectional,
      label: newMapping.label,
    };

    const result = await createMapping(request);
    if (result) {
      setShowCreateForm(false);
      setNewMapping({
        midi_channel: 0,
        midi_cc: 7,
        taper_curve: 'linear',
        min_value: 0,
        max_value: 1,
        invert: false,
        bidirectional: false,
      });
    }
  };

  const handleUpdateMapping = async (id: number, updates: Partial<UpdateMappingRequest>) => {
    const request: UpdateMappingRequest = {
      id,
      ...updates,
    };
    await updateMapping(request);
  };

  const handleDeleteMapping = async (id: number) => {
    await deleteMapping(id);
  };

  const handleParameterSelect = (parameterId: number, parameterName: string, deviceId: number) => {
    setNewMapping({
      ...newMapping,
      ucnet_parameter_id: parameterId,
      ucnet_parameter_name: parameterName,
      ucnet_device_id: deviceId,
    });
  };

  // Empty state when no project is selected
  if (!projectId) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <svg
            className="w-16 h-16 text-slate-600 mx-auto mb-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
            />
          </svg>
          <p className="text-slate-400 text-lg">No project selected</p>
          <p className="text-slate-500 text-sm mt-2">Select or create a project to manage mappings</p>
        </div>
      </div>
    );
  }

  // Empty state when no devices are connected
  if (!loading && midiDevices.length === 0 && ucnetDevices.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <svg
            className="w-16 h-16 text-slate-600 mx-auto mb-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
            />
          </svg>
          <p className="text-slate-400 text-lg">No devices connected</p>
          <p className="text-slate-500 text-sm mt-2">Connect MIDI and UCNet devices to create mappings</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-white">Parameter Mappings</h2>
          <p className="text-slate-400 text-sm mt-1">
            Map MIDI controls to UCNet parameters
          </p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={refresh}
            disabled={loading}
            className="px-4 py-2 bg-slate-700 hover:bg-slate-600 disabled:bg-slate-800 disabled:text-slate-600 text-white font-semibold rounded-lg transition-colors"
            aria-label="Refresh mappings"
          >
            {loading ? 'Loading...' : 'Refresh'}
          </button>
          <button
            onClick={() => setShowCreateForm(!showCreateForm)}
            className="px-4 py-2 bg-cyan-500 hover:bg-cyan-600 text-slate-950 font-semibold rounded-lg transition-colors"
            aria-label="Create new mapping"
          >
            {showCreateForm ? 'Cancel' : 'New Mapping'}
          </button>
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="p-4 bg-rose-900/20 border border-rose-800 rounded-md">
          <p className="text-rose-400 text-sm">{error}</p>
        </div>
      )}

      {/* Create Mapping Form */}
      {showCreateForm && (
        <div className="p-6 bg-slate-800 border border-slate-700 rounded-lg space-y-4">
          <h3 className="text-lg font-semibold text-white">Create New Mapping</h3>
          
          <div className="grid grid-cols-2 gap-4">
            {/* MIDI Device */}
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                MIDI Device
              </label>
              <select
                value={newMapping.midi_device_id ?? ''}
                onChange={(e) => setNewMapping({ ...newMapping, midi_device_id: parseInt(e.target.value) })}
                className="w-full px-3 py-2 bg-slate-900 border border-slate-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-cyan-500"
              >
                <option value="">Select MIDI device...</option>
                {midiDevices.map(device => (
                  <option key={device.id} value={device.id}>
                    {device.device_name}
                  </option>
                ))}
              </select>
            </div>

            {/* MIDI Channel */}
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                MIDI Channel
              </label>
              <select
                value={newMapping.midi_channel ?? 0}
                onChange={(e) => setNewMapping({ ...newMapping, midi_channel: parseInt(e.target.value) })}
                className="w-full px-3 py-2 bg-slate-900 border border-slate-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-cyan-500"
              >
                {Array.from({ length: 16 }, (_, i) => (
                  <option key={i} value={i}>
                    Channel {i + 1}
                  </option>
                ))}
              </select>
            </div>

            {/* MIDI CC */}
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                MIDI CC Number
              </label>
              <div className="flex gap-2">
                <input
                  type="number"
                  value={newMapping.midi_cc ?? 7}
                  onChange={(e) => setNewMapping({ ...newMapping, midi_cc: parseInt(e.target.value) })}
                  min="0"
                  max="127"
                  className="flex-1 px-3 py-2 bg-slate-900 border border-slate-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-cyan-500"
                />
                <button
                  type="button"
                  onClick={() => {
                    if (newMapping.ucnet_device_id && newMapping.ucnet_parameter_name) {
                      // Start learning with the selected UCNet parameter
                      startLearn(
                        String(newMapping.ucnet_device_id),
                        1, // Default channel
                        'volume' // Default parameter type
                      );
                    }
                  }}
                  disabled={isLearning || !newMapping.ucnet_device_id}
                  className={`px-3 py-2 rounded text-sm font-medium transition-all ${
                    isLearning
                      ? 'bg-amber-500 text-slate-900 animate-pulse'
                      : 'bg-slate-700 text-slate-300 hover:bg-slate-600 hover:text-white disabled:bg-slate-800 disabled:text-slate-600'
                  }`}
                  title={
                    !newMapping.ucnet_device_id
                      ? 'Select a UCNet parameter first'
                      : isLearning
                      ? 'Listening for MIDI input...'
                      : 'Click to learn MIDI CC from controller'
                  }
                >
                  {isLearning ? (
                    <span className="flex items-center gap-1">
                      <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                      </svg>
                      Learning...
                    </span>
                  ) : (
                    <span className="flex items-center gap-1">
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15.536a5 5 0 001.414 1.414m2.828-9.9a9 9 0 012.828 2.828" />
                      </svg>
                      Learn
                    </span>
                  )}
                </button>
              </div>
              {isLearning && (
                <div className="flex items-center justify-between mt-1">
                  <p className="text-xs text-amber-400">
                    Move a MIDI control to capture its CC number...
                  </p>
                  <button
                    type="button"
                    onClick={cancelLearn}
                    className="text-xs text-slate-400 hover:text-white underline"
                  >
                    Cancel
                  </button>
                </div>
              )}
            </div>

            {/* UCNet Parameter */}
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                UCNet Parameter
              </label>
              <ParameterSelector
                devices={devices}
                value={newMapping.ucnet_parameter_id ?? null}
                onChange={handleParameterSelect}
              />
            </div>
          </div>

          {/* Advanced Options */}
          <details className="group">
            <summary className="cursor-pointer text-sm font-medium text-cyan-400 hover:text-cyan-300">
              Advanced Options
            </summary>
            <div className="mt-4 grid grid-cols-2 gap-4">
              {/* Taper Curve */}
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  Taper Curve
                </label>
                <select
                  value={newMapping.taper_curve ?? 'linear'}
                  onChange={(e) => setNewMapping({ ...newMapping, taper_curve: e.target.value as any })}
                  className="w-full px-3 py-2 bg-slate-900 border border-slate-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-cyan-500"
                >
                  <option value="linear">Linear</option>
                  <option value="logarithmic">Logarithmic</option>
                  <option value="exponential">Exponential</option>
                  <option value="s-curve">S-Curve</option>
                </select>
              </div>

              {/* Label */}
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  Label (Optional)
                </label>
                <input
                  type="text"
                  value={newMapping.label ?? ''}
                  onChange={(e) => setNewMapping({ ...newMapping, label: e.target.value })}
                  placeholder="Custom label..."
                  className="w-full px-3 py-2 bg-slate-900 border border-slate-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-cyan-500"
                />
              </div>

              {/* Min Value */}
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  Min Value
                </label>
                <input
                  type="number"
                  value={newMapping.min_value ?? 0}
                  onChange={(e) => setNewMapping({ ...newMapping, min_value: parseFloat(e.target.value) })}
                  step="0.01"
                  min="0"
                  max="1"
                  className="w-full px-3 py-2 bg-slate-900 border border-slate-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-cyan-500"
                />
              </div>

              {/* Max Value */}
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  Max Value
                </label>
                <input
                  type="number"
                  value={newMapping.max_value ?? 1}
                  onChange={(e) => setNewMapping({ ...newMapping, max_value: parseFloat(e.target.value) })}
                  step="0.01"
                  min="0"
                  max="1"
                  className="w-full px-3 py-2 bg-slate-900 border border-slate-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-cyan-500"
                />
              </div>

              {/* Checkboxes */}
              <div className="col-span-2 flex gap-6">
                <label className="flex items-center gap-2 text-sm text-slate-300 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={newMapping.invert ?? false}
                    onChange={(e) => setNewMapping({ ...newMapping, invert: e.target.checked })}
                    className="w-4 h-4 bg-slate-900 border-slate-700 rounded text-cyan-500 focus:ring-2 focus:ring-cyan-500"
                  />
                  Invert
                </label>
                <label className="flex items-center gap-2 text-sm text-slate-300 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={newMapping.bidirectional ?? false}
                    onChange={(e) => setNewMapping({ ...newMapping, bidirectional: e.target.checked })}
                    className="w-4 h-4 bg-slate-900 border-slate-700 rounded text-cyan-500 focus:ring-2 focus:ring-cyan-500"
                  />
                  Bidirectional
                </label>
              </div>
            </div>
          </details>

          {/* Actions */}
          <div className="flex justify-end gap-3 pt-4 border-t border-slate-700">
            <button
              onClick={() => setShowCreateForm(false)}
              className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleCreateMapping}
              className="px-4 py-2 bg-cyan-500 hover:bg-cyan-600 text-slate-950 font-semibold rounded-lg transition-colors"
            >
              Create Mapping
            </button>
          </div>
        </div>
      )}

      {/* Mappings List */}
      <div className="space-y-3">
        {loading && mappings.length === 0 ? (
          <div className="text-center py-12">
            <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-cyan-500"></div>
            <p className="text-slate-400 mt-4">Loading mappings...</p>
          </div>
        ) : mappings.length === 0 ? (
          <div className="text-center py-12 bg-slate-900 border border-slate-800 rounded-lg">
            <svg
              className="w-12 h-12 text-slate-600 mx-auto mb-3"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M13 10V3L4 14h7v7l9-11h-7z"
              />
            </svg>
            <p className="text-slate-400">No mappings yet</p>
            <p className="text-slate-500 text-sm mt-1">
              Click "New Mapping" to create your first parameter mapping
            </p>
          </div>
        ) : (
          <>
            <div className="text-sm text-slate-400 mb-2">
              {mappings.length} {mappings.length === 1 ? 'mapping' : 'mappings'}
            </div>
            {mappings.map(mapping => (
              <MappingRow
                key={mapping.id}
                mapping={mapping}
                devices={devices}
                onUpdate={handleUpdateMapping}
                onDelete={handleDeleteMapping}
              />
            ))}

            {/* Visual Feedback Section */}
            <VisualFeedbackSection mappings={mappings} />
          </>
        )}
      </div>
    </div>
  );
}

/**
 * Visual feedback section showing MixerStrips for mapped channels.
 * Displays real-time parameter values from sync events.
 */
interface VisualFeedbackSectionProps {
  mappings: Mapping[];
}

function VisualFeedbackSection({ mappings }: VisualFeedbackSectionProps) {
  // Extract unique channels from mappings
  const channels = useMemo(() => extractUniqueChannels(mappings), [mappings]);

  if (channels.length === 0) {
    return null;
  }

  return (
    <div className="mt-8 pt-6 border-t border-slate-700">
      <div className="mb-4">
        <h3 className="text-lg font-semibold text-white">Visual Feedback</h3>
        <p className="text-xs text-slate-400 mt-1">
          Real-time parameter values for mapped channels
        </p>
      </div>

      <div 
        className="flex gap-4 overflow-x-auto pb-4"
        role="region"
        aria-label="Mixer channel strips"
      >
        {channels.map((channel) => (
          <MixerStrip
            key={channel.key}
            channelNumber={channel.channelNumber}
            label={channel.label}
            parameterPrefix={channel.parameterPrefix}
          />
        ))}
      </div>
    </div>
  );
}
