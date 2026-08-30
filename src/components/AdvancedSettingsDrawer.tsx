import React from "react";
import { Sliders, X, Zap, Shield, Image, Cpu, Volume2, Monitor } from "lucide-react";
import { AdvancedSettings, HardwareInfo } from "../types";

interface AdvancedSettingsDrawerProps {
    isOpen: boolean;
    onClose: () => void;
    settings: AdvancedSettings;
    onChangeSettings: (newSettings: AdvancedSettings) => void;
    hardware: HardwareInfo | null;
}

export const AdvancedSettingsDrawer: React.FC<AdvancedSettingsDrawerProps> = ({
    isOpen,
    onClose,
    settings,
    onChangeSettings,
    hardware,
}) => {
    if (!isOpen) return null;

    const updateSetting = <K extends keyof AdvancedSettings>(
        key: K,
        val: AdvancedSettings[K]
    ) => {
        onChangeSettings({
            ...settings,
            [key]: val,
        });
    };

    const resolutionOptions = [
        { label: "Original (No Scaling)", value: "original" },
        { label: "4K UHD (3840x2160)", value: "3840x2160" },
        { label: "Full HD (1920x1080)", value: "1920x1080" },
        { label: "HD (1280x720)", value: "1280x720" },
        { label: "SD (854x480)", value: "854x480" },
    ];

    const audioBitrates = ["320k", "256k", "192k", "128k", "96k", "64k"];

    return (
        <div className="fixed inset-0 z-50 overflow-hidden flex justify-end">
            {/* Backdrop */}
            <div
                className="fixed inset-0 bg-black/60 backdrop-blur-sm transition-opacity"
                onClick={onClose}
            />

            {/* Drawer content */}
            <div className="relative w-full max-w-md bg-surface border-l border-border h-full flex flex-col shadow-2xl z-10">
                {/* Header */}
                <div className="px-6 py-4 border-b border-border flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <Sliders className="w-5 h-5 text-brand-400" />
                        <h2 className="font-bold text-slate-100">Advanced Conversion Settings</h2>
                    </div>
                    <button
                        onClick={onClose}
                        className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-surface-raised transition-colors"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                {/* Scrollable controls */}
                <div className="flex-1 overflow-y-auto p-6 space-y-6 text-xs">
                    {/* CRF Quality Slider */}
                    <div className="space-y-2">
                        <div className="flex items-center justify-between">
                            <label className="font-semibold text-slate-200 flex items-center gap-1.5">
                                <Image className="w-3.5 h-3.5 text-brand-400" />
                                CRF Quality / Rate Control
                            </label>
                            <span className="font-mono bg-surface-raised px-2 py-0.5 rounded border border-border text-brand-300 font-bold">
                                CRF {settings.crf}
                            </span>
                        </div>
                        <input
                            type="range"
                            min="0"
                            max="51"
                            value={settings.crf}
                            onChange={(e) => updateSetting("crf", parseInt(e.target.value))}
                            className="w-full h-1.5 bg-surface-raised rounded-lg appearance-none cursor-pointer accent-brand-500"
                        />
                        <div className="flex justify-between text-[10px] text-slate-400">
                            <span>0 (Lossless)</span>
                            <span>18-23 (Visually Lossless)</span>
                            <span>28 (Balanced)</span>
                            <span>51 (Max Compression)</span>
                        </div>
                    </div>

                    {/* Resolution Lock */}
                    <div className="space-y-2">
                        <label className="font-semibold text-slate-200 flex items-center gap-1.5">
                            <Monitor className="w-3.5 h-3.5 text-brand-400" />
                            Resolution Lock / Downscaling
                        </label>
                        <select
                            value={settings.resolution}
                            onChange={(e) => updateSetting("resolution", e.target.value)}
                            className="w-full bg-surface-raised border border-border rounded-lg px-3 py-2 text-slate-200 focus:outline-none focus:border-brand-500"
                        >
                            {resolutionOptions.map((res) => (
                                <option key={res.value} value={res.value}>
                                    {res.label}
                                </option>
                            ))}
                        </select>
                        <p className="text-[11px] text-slate-400">
                            Downscales media to target resolution while preserving aspect ratio.
                        </p>
                    </div>

                    {/* Hardware Acceleration Toggle */}
                    <div className="bg-surface-raised rounded-xl p-4 border border-border space-y-3">
                        <div className="flex items-center justify-between">
                            <div className="space-y-0.5">
                                <span className="font-semibold text-slate-200 flex items-center gap-1.5">
                                    <Zap className="w-3.5 h-3.5 text-amber-400" />
                                    GPU Hardware Acceleration
                                </span>
                                <p className="text-[11px] text-slate-400">
                                    {hardware?.hardware_acceleration_supported
                                        ? `Enabled via ${hardware.gpu_vendor || "GPU"}`
                                        : "No dedicated GPU detected; using CPU multi-threading"}
                                </p>
                            </div>
                            <input
                                type="checkbox"
                                checked={settings.hardwareAccel}
                                disabled={!hardware?.hardware_acceleration_supported}
                                onChange={(e) => updateSetting("hardwareAccel", e.target.checked)}
                                className="w-4 h-4 rounded accent-brand-500 cursor-pointer"
                            />
                        </div>

                        {/* Encoder Selection */}
                        {hardware && hardware.available_encoders.length > 0 && (
                            <div className="space-y-1.5 pt-2 border-t border-border/60">
                                <label className="font-medium text-slate-300 flex items-center gap-1">
                                    <Cpu className="w-3 h-3 text-brand-400" />
                                    Encoder Profile
                                </label>
                                <select
                                    value={settings.selectedEncoder}
                                    onChange={(e) => updateSetting("selectedEncoder", e.target.value)}
                                    className="w-full bg-surface border border-border rounded-lg px-3 py-1.5 text-slate-200 focus:outline-none focus:border-brand-500"
                                >
                                    <option value="auto">Auto-detect optimal encoder</option>
                                    {hardware.available_encoders.map((enc) => (
                                        <option key={enc.id} value={enc.id}>
                                            {enc.name} {enc.is_hardware ? "(GPU)" : "(CPU)"}
                                        </option>
                                    ))}
                                </select>
                            </div>
                        )}
                    </div>

                    {/* Privacy & Metadata Stripping */}
                    <div className="bg-surface-raised rounded-xl p-4 border border-border flex items-center justify-between">
                        <div className="space-y-0.5 pr-2">
                            <span className="font-semibold text-slate-200 flex items-center gap-1.5">
                                <Shield className="w-3.5 h-3.5 text-emerald-400" />
                                Strip Metadata & EXIF
                            </span>
                            <p className="text-[11px] text-slate-400">
                                Removes GPS coordinates, camera models, author tags, and timestamps for anonymity.
                            </p>
                        </div>
                        <input
                            type="checkbox"
                            checked={settings.stripMetadata}
                            onChange={(e) => updateSetting("stripMetadata", e.target.checked)}
                            className="w-4 h-4 rounded accent-brand-500 cursor-pointer"
                        />
                    </div>

                    {/* Audio Bitrate */}
                    <div className="space-y-2">
                        <label className="font-semibold text-slate-200 flex items-center gap-1.5">
                            <Volume2 className="w-3.5 h-3.5 text-brand-400" />
                            Audio Bitrate
                        </label>
                        <div className="grid grid-cols-3 gap-2">
                            {audioBitrates.map((rate) => (
                                <button
                                    key={rate}
                                    type="button"
                                    onClick={() => updateSetting("audioBitrate", rate)}
                                    className={`py-1.5 px-2 rounded-lg border text-center font-mono text-xs transition-colors ${settings.audioBitrate === rate
                                            ? "bg-brand-500/20 border-brand-500 text-brand-300 font-bold"
                                            : "bg-surface-raised border-border text-slate-300 hover:bg-surface hover:text-white"
                                        }`}
                                >
                                    {rate}
                                </button>
                            ))}
                        </div>
                    </div>
                </div>

                {/* Footer */}
                <div className="p-4 border-t border-border bg-surface flex justify-end">
                    <button
                        onClick={onClose}
                        className="px-4 py-2 rounded-xl bg-brand-600 hover:bg-brand-500 text-white font-semibold text-xs transition-colors"
                    >
                        Apply & Close
                    </button>
                </div>
            </div>
        </div>
    );
};
