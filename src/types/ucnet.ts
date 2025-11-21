/**
 * UCNet device types for frontend
 */

export type ConnectionType = "Network" | "Usb";

export type ConnectionState =
  | "Discovered"
  | "Connecting"
  | "Connected"
  | "Disconnected"
  | "Failed";

export interface UcNetDevice {
  id: string;
  model: string;
  firmware_version: string;
  connection_type: ConnectionType;
  state: ConnectionState;
  identifier: string;
}
