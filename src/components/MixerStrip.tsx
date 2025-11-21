import { useParameterValue } from '../hooks/useParameterValue';
import { Fader } from './Fader';
import { MuteButton } from './MuteButton';
import { PanKnob } from './PanKnob';

export interface MixerStripProps {
  /** Channel number (1-based) */
  channelNumber: number;
  /** Channel label */
  label?: string;
  /** UCNet parameter prefix (e.g., "line/ch1") */
  parameterPrefix: string;
}

/**
 * MixerStrip component combines fader, mute button, and pan knob
 * for a single mixer channel with real-time parameter updates.
 */
export function MixerStrip({ channelNumber, label, parameterPrefix }: MixerStripProps) {
  const channelLabel = label || `Ch ${channelNumber}`;

  // Volume parameter
  const volume = useParameterValue({
    parameter: `${parameterPrefix}/vol`,
    initialValue: 0.75, // -12dB default
  });

  // Mute parameter (0 = unmuted, 1 = muted)
  const mute = useParameterValue({
    parameter: `${parameterPrefix}/mute`,
    initialValue: 0,
  });

  // Pan parameter (0 = full left, 0.5 = center, 1 = full right)
  const pan = useParameterValue({
    parameter: `${parameterPrefix}/pan`,
    initialValue: 0.5,
  });

  const handleMuteToggle = () => {
    mute.setValue(mute.value === 0 ? 1 : 0);
  };

  return (
    <div className="flex flex-col items-center gap-4 p-4 bg-slate-900 rounded-lg border border-slate-800">
      {/* Channel Label */}
      <div className="text-sm font-semibold text-slate-200 text-center min-w-[5rem]">
        {channelLabel}
      </div>

      {/* Pan Knob */}
      <PanKnob
        value={pan.value}
        onChange={pan.setValue}
        label="Pan"
        isActive={pan.isActive}
      />

      {/* Fader */}
      <Fader
        value={volume.value}
        onChange={volume.setValue}
        label="Volume"
        isActive={volume.isActive}
        height={200}
      />

      {/* Mute Button */}
      <MuteButton
        muted={mute.value > 0.5}
        onToggle={handleMuteToggle}
        label="Mute"
        isActive={mute.isActive}
      />
    </div>
  );
}
