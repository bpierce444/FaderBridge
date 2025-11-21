/**
 * Translation types for MIDI to UCNet parameter mapping
 * These types mirror the Rust backend types for type safety
 */

/**
 * UCNet parameter types that can be controlled
 */
export type UcNetParameterType = 'volume' | 'mute' | 'pan';

/**
 * UCNet parameter value (float for continuous, boolean for toggle)
 */
export type UcNetParameterValue = number | boolean;

/**
 * Taper curve type for fader response
 */
export type TaperCurve = 'linear' | 'logarithmic' | 'audiotaper';

/**
 * A mapping between a MIDI control and a UCNet parameter
 */
export interface ParameterMapping {
  /** MIDI channel (0-15) */
  midi_channel: number;
  /** MIDI controller number (for CC messages) */
  midi_controller?: number;
  /** MIDI note number (for Note On/Off messages) */
  midi_note?: number;
  /** Target UCNet device ID */
  ucnet_device_id: string;
  /** Target UCNet channel number (1-based) */
  ucnet_channel: number;
  /** Target parameter type */
  parameter_type: UcNetParameterType;
  /** Taper curve for continuous parameters */
  taper_curve: TaperCurve;
  /** Whether to use 14-bit MIDI CC (MSB/LSB pairs) */
  use_14bit: boolean;
  /** MSB controller number for 14-bit mode */
  midi_controller_msb?: number;
  /** LSB controller number for 14-bit mode */
  midi_controller_lsb?: number;
}

/**
 * Result of a parameter mapping operation
 */
export interface MappingResult {
  /** Target UCNet device ID */
  device_id: string;
  /** Target channel number */
  channel: number;
  /** Parameter type being controlled */
  parameter_type: UcNetParameterType;
  /** Mapped parameter value */
  value: UcNetParameterValue;
}

/**
 * Helper functions for creating parameter mappings
 */
export const MappingHelpers = {
  /**
   * Creates a new volume mapping from MIDI CC to UCNet channel
   */
  newVolume(
    midiChannel: number,
    midiController: number,
    ucnetDeviceId: string,
    ucnetChannel: number,
    taperCurve: TaperCurve = 'audiotaper'
  ): ParameterMapping {
    return {
      midi_channel: midiChannel,
      midi_controller: midiController,
      ucnet_device_id: ucnetDeviceId,
      ucnet_channel: ucnetChannel,
      parameter_type: 'volume',
      taper_curve: taperCurve,
      use_14bit: false,
    };
  },

  /**
   * Creates a new 14-bit volume mapping from MIDI CC MSB/LSB to UCNet channel
   */
  newVolume14Bit(
    midiChannel: number,
    midiControllerMsb: number,
    midiControllerLsb: number,
    ucnetDeviceId: string,
    ucnetChannel: number,
    taperCurve: TaperCurve = 'audiotaper'
  ): ParameterMapping {
    return {
      midi_channel: midiChannel,
      ucnet_device_id: ucnetDeviceId,
      ucnet_channel: ucnetChannel,
      parameter_type: 'volume',
      taper_curve: taperCurve,
      use_14bit: true,
      midi_controller_msb: midiControllerMsb,
      midi_controller_lsb: midiControllerLsb,
    };
  },

  /**
   * Creates a new mute mapping from MIDI Note to UCNet channel
   */
  newMute(
    midiChannel: number,
    midiNote: number,
    ucnetDeviceId: string,
    ucnetChannel: number
  ): ParameterMapping {
    return {
      midi_channel: midiChannel,
      midi_note: midiNote,
      ucnet_device_id: ucnetDeviceId,
      ucnet_channel: ucnetChannel,
      parameter_type: 'mute',
      taper_curve: 'linear',
      use_14bit: false,
    };
  },

  /**
   * Creates a new pan mapping from MIDI CC to UCNet channel
   */
  newPan(
    midiChannel: number,
    midiController: number,
    ucnetDeviceId: string,
    ucnetChannel: number
  ): ParameterMapping {
    return {
      midi_channel: midiChannel,
      midi_controller: midiController,
      ucnet_device_id: ucnetDeviceId,
      ucnet_channel: ucnetChannel,
      parameter_type: 'pan',
      taper_curve: 'linear',
      use_14bit: false,
    };
  },
};
