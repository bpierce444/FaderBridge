/**
 * MIDI device types and interfaces
 * These types match the Rust backend types for seamless serialization
 */

/**
 * Type of MIDI device
 */
export type MidiDeviceType = 'input' | 'output';

/**
 * Connection status of a MIDI device
 */
export type MidiConnectionStatus = 'available' | 'connected' | 'disconnected';

/**
 * Represents a MIDI device (input or output)
 */
export interface MidiDevice {
  /** Unique identifier for this device */
  id: string;
  /** Human-readable name of the device */
  name: string;
  /** Manufacturer name (if available) */
  manufacturer?: string;
  /** Device type (input or output) */
  device_type: MidiDeviceType;
  /** Port number in the system */
  port_number: number;
  /** Connection status */
  status: MidiConnectionStatus;
}

/**
 * MIDI message types for control surfaces
 */
export type MidiMessageType =
  | { type: 'control_change'; channel: number; controller: number; value: number }
  | { type: 'note_on'; channel: number; note: number; velocity: number }
  | { type: 'note_off'; channel: number; note: number; velocity: number }
  | { type: 'pitch_bend'; channel: number; value: number }
  | { type: 'program_change'; channel: number; program: number };

/**
 * Result of checking for device changes
 */
export interface MidiDeviceChanges {
  added: MidiDevice[];
  removed: MidiDevice[];
}
