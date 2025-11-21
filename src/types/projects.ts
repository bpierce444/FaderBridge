/**
 * TypeScript types for project management
 * These mirror the Rust types in src-tauri/src/db/types.rs
 */

export interface Project {
  id: number;
  name: string;
  description: string | null;
  created_at: string;
  updated_at: string;
  last_opened_at: string | null;
  is_active: boolean;
}

export interface CreateProjectRequest {
  name: string;
  description?: string;
}

export interface UpdateProjectRequest {
  id: number;
  name?: string;
  description?: string;
}

export type DeviceType = 'midi' | 'ucnet';

export type ConnectionType = 'usb' | 'network' | 'virtual';

export interface Device {
  id: number;
  project_id: number;
  device_type: DeviceType;
  device_name: string;
  device_id: string;
  connection_type: ConnectionType | null;
  config_json: string | null;
  created_at: string;
}

export interface CreateDeviceRequest {
  project_id: number;
  device_type: DeviceType;
  device_name: string;
  device_id: string;
  connection_type?: ConnectionType;
  config_json?: string;
}

export type TaperCurve = 'linear' | 'logarithmic' | 'exponential' | 's-curve';

export interface Mapping {
  id: number;
  project_id: number;
  midi_device_id: number;
  ucnet_device_id: number;
  midi_channel: number;
  midi_cc: number;
  ucnet_parameter_id: number;
  ucnet_parameter_name: string;
  taper_curve: TaperCurve;
  min_value: number;
  max_value: number;
  invert: boolean;
  bidirectional: boolean;
  label: string | null;
  created_at: string;
}

export interface CreateMappingRequest {
  project_id: number;
  midi_device_id: number;
  ucnet_device_id: number;
  midi_channel: number;
  midi_cc: number;
  ucnet_parameter_id: number;
  ucnet_parameter_name: string;
  taper_curve?: TaperCurve;
  min_value?: number;
  max_value?: number;
  invert?: boolean;
  bidirectional?: boolean;
  label?: string;
}

export interface UpdateMappingRequest {
  id: number;
  taper_curve?: TaperCurve;
  min_value?: number;
  max_value?: number;
  invert?: boolean;
  bidirectional?: boolean;
  label?: string;
}

export interface ProjectExport {
  version: string;
  exported_at: string;
  project: Project;
  devices: Device[];
  mappings: Mapping[];
}
