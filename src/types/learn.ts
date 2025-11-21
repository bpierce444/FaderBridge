/**
 * TypeScript types for MIDI Learn functionality
 * These types match the Rust backend types in src-tauri/src/commands/learn.rs
 */

import { ParameterMapping, UcNetParameterType } from './mapping';

/**
 * MIDI Learn state
 */
export type LearnState =
  | { type: 'idle' }
  | {
      type: 'listening';
      device_id: string;
      channel: number;
      parameter_type: UcNetParameterType;
      elapsed_ms: number;
    };

/**
 * MIDI Learn result
 */
export type LearnResult =
  | { type: 'success'; mapping: ParameterMapping }
  | { type: 'timeout' }
  | { type: 'cancelled' }
  | { type: 'waiting' };

/**
 * Helper to check if in learn mode
 */
export function isLearning(state: LearnState): boolean {
  return state.type === 'listening';
}

/**
 * Helper to get elapsed time in seconds
 */
export function getElapsedSeconds(state: LearnState): number {
  if (state.type === 'listening') {
    return state.elapsed_ms / 1000;
  }
  return 0;
}

/**
 * Helper to get remaining time in seconds (10 second timeout)
 */
export function getRemainingSeconds(state: LearnState): number {
  if (state.type === 'listening') {
    const elapsed = state.elapsed_ms / 1000;
    return Math.max(0, 10 - elapsed);
  }
  return 0;
}
