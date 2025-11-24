/**
 * ParameterSelector component
 * Allows users to select a UCNet parameter for mapping
 */

import { Device } from '../hooks/useMappings';

export interface UcNetParameter {
  id: number;
  name: string;
  device_id: number;
  device_name: string;
}

export interface ParameterSelectorProps {
  /** Available UCNet devices */
  devices: Device[];
  /** Currently selected parameter ID */
  value: number | null;
  /** Callback when parameter is selected */
  onChange: (parameterId: number, parameterName: string, deviceId: number) => void;
  /** Optional CSS class name */
  className?: string;
  /** Whether the selector is disabled */
  disabled?: boolean;
}

/**
 * Generates a list of available UCNet parameters from devices
 * For MVP, we support: Volume, Mute, Pan for channels 1-8
 */
function generateParameters(devices: Device[]): UcNetParameter[] {
  const parameters: UcNetParameter[] = [];
  
  const ucnetDevices = devices.filter(d => d.device_type === 'ucnet');
  
  for (const device of ucnetDevices) {
    // Generate parameters for 8 channels
    for (let ch = 1; ch <= 8; ch++) {
      // Volume parameter
      parameters.push({
        id: parameters.length + 1,
        name: `Channel ${ch} Volume`,
        device_id: device.id,
        device_name: device.device_name,
      });
      
      // Mute parameter
      parameters.push({
        id: parameters.length + 1,
        name: `Channel ${ch} Mute`,
        device_id: device.id,
        device_name: device.device_name,
      });
      
      // Pan parameter
      parameters.push({
        id: parameters.length + 1,
        name: `Channel ${ch} Pan`,
        device_id: device.id,
        device_name: device.device_name,
      });
    }
  }
  
  return parameters;
}

/**
 * Component for selecting UCNet parameters
 * 
 * @example
 * ```tsx
 * <ParameterSelector
 *   devices={devices}
 *   value={selectedParameterId}
 *   onChange={(id, name, deviceId) => handleParameterChange(id, name, deviceId)}
 * />
 * ```
 */
export function ParameterSelector({
  devices,
  value,
  onChange,
  className = '',
  disabled = false,
}: ParameterSelectorProps) {
  const parameters = generateParameters(devices);
  
  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const selectedId = parseInt(e.target.value, 10);
    const parameter = parameters.find(p => p.id === selectedId);
    
    if (parameter) {
      onChange(parameter.id, parameter.name, parameter.device_id);
    }
  };

  if (parameters.length === 0) {
    return (
      <div className={`text-slate-400 text-sm italic ${className}`}>
        No UCNet devices available
      </div>
    );
  }

  // Group parameters by device
  const parametersByDevice = parameters.reduce<Record<string, UcNetParameter[]>>((acc, param) => {
    if (!acc[param.device_name]) {
      acc[param.device_name] = [];
    }
    acc[param.device_name]!.push(param);
    return acc;
  }, {});

  return (
    <select
      value={value ?? ''}
      onChange={handleChange}
      disabled={disabled}
      className={`
        w-full px-3 py-2 
        bg-slate-800 border border-slate-700 
        text-white rounded 
        focus:outline-none focus:ring-2 focus:ring-cyan-500 focus:border-transparent
        disabled:opacity-50 disabled:cursor-not-allowed
        ${className}
      `}
      aria-label="Select UCNet parameter"
    >
      <option value="">Select a parameter...</option>
      {Object.entries(parametersByDevice).map(([deviceName, params]) => (
        <optgroup key={deviceName} label={deviceName}>
          {params.map(param => (
            <option key={param.id} value={param.id}>
              {param.name}
            </option>
          ))}
        </optgroup>
      ))}
    </select>
  );
}
