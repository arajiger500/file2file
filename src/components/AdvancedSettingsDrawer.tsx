import React from "react";
import { X } from "lucide-react";
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
        { label: "Original resolution", value: "original" },
        { label: "4K UHD (3840x2160)", value: "3840x2160" },
        { label: "Full HD (1920x1080)", value: "1920x1080" },
        { label: "HD (1280x720)", value: "1280x720" },
        { label: "SD (854x480)", value: "854x480" },
    ];

    const audioBitrates = ["320k", "256k", "192k", "128k", "96k", "64k"];

    return (
        <div className="fixed inset-0 z-50 flex justify-end">
            <div className="fixed inset-0 bg-black/60 backdrop-blur-[2px]" onClick={onClose} />

            <div className="relative w-full max-w-[400px] bg-surface border-l border-border h-full flex flex-col z-10 shadow-2xl">
                <div className="h-[56px] px-6 border-b border-border bg-surface flex items-center justify-between">
                    <h2 className="text-[15px] font-semibold text-text-primary">Settings</h2>
                    <button onClick={onClose} className="p-2 -mr-2 text-text-muted hover:text-text-primary transition-colors">
                        <X className="w-5 h-5" />
                    </button>
                </div>

                <div className="flex-1 overflow-y-auto p-6 space-y-8 custom-scrollbar">
                    {/* Video Section */}
                    <div className="space-y-6">
                        <h3 className="text-tiny font-bold uppercase tracking-widest text-text-muted">Video</h3>

                        <div className="space-y-4">
                            <div className="flex items-center justify-between">
                                <label className="text-[13px] font-medium text-text-secondary">Quality (CRF)</label>
                                <span className="text-[13px] font-mono text-accent">{settings.crf}</span>
                            </div>
                            <input
                                type="range"
                                min="0"
                                max="51"
                                value={settings.crf}
                                onChange={(e) => updateSetting("crf", parseInt(e.target.value))}
                                className="w-full h-1 bg-border rounded-full appearance-none cursor-pointer accent-accent"
                            />
                            <div className="flex justify-between text-tiny font-mono text-text-disabled uppercase">
                                <span>Higher Quality</span>
                                <span>Smaller File</span>
                            </div>
                        </div>

                        <div className="space-y-3">
                            <label className="text-[13px] font-medium text-text-secondary">Resolution</label>
                            <select
                                value={settings.resolution}
                                onChange={(e) => updateSetting("resolution", e.target.value)}
                                className="input-field w-full h-[38px] cursor-pointer"
                            >
                                {resolutionOptions.map((res) => (
                                    <option key={res.value} value={res.value}>{res.label}</option>
                                ))}
                            </select>
                        </div>
                    </div>

                    <div className="h-[1px] bg-border" />

                    {/* Processing Section */}
                    <div className="space-y-6">
                        <h3 className="text-tiny font-bold uppercase tracking-widest text-text-muted">Processing</h3>

                        <div className="flex items-center justify-between">
                            <div className="space-y-0.5">
                                <label className="text-[13px] font-medium text-text-secondary">Hardware acceleration</label>
                                <p className="text-tiny text-text-muted">Use GPU if available</p>
                            </div>
                            <button
                                onClick={() => updateSetting("hardwareAccel", !settings.hardwareAccel)}
                                disabled={!hardware?.hardware_acceleration_supported}
                                className={`w-9 h-5 rounded-full transition-all relative ${
                                    settings.hardwareAccel ? "bg-accent" : "bg-border"
                                }`}
                            >
                                <div className={`absolute top-1 w-3 h-3 rounded-full bg-white transition-all ${settings.hardwareAccel ? "left-[22px]" : "left-1"}`} />
                            </button>
                        </div>

                        {hardware && hardware.available_encoders.length > 0 && (
                            <div className="space-y-3">
                                <label className="text-[13px] font-medium text-text-secondary">Encoder</label>
                                <select
                                    value={settings.selectedEncoder}
                                    onChange={(e) => updateSetting("selectedEncoder", e.target.value)}
                                    className="input-field w-full h-[38px] cursor-pointer"
                                >
                                    <option value="auto">Auto-detect</option>
                                    {hardware.available_encoders.map((enc) => (
                                        <option key={enc.id} value={enc.id}>{enc.name}</option>
                                    ))}
                                </select>
                            </div>
                        )}
                    </div>

                    <div className="h-[1px] bg-border" />

                    {/* Privacy Section */}
                    <div className="space-y-6">
                        <h3 className="text-tiny font-bold uppercase tracking-widest text-text-muted">Privacy</h3>

                        <div className="flex items-center justify-between">
                            <div className="space-y-0.5">
                                <label className="text-[13px] font-medium text-text-secondary">Remove metadata</label>
                                <p className="text-tiny text-text-muted">Strip GPS and device tags</p>
                            </div>
                            <button
                                onClick={() => updateSetting("stripMetadata", !settings.stripMetadata)}
                                className={`w-9 h-5 rounded-full transition-all relative ${
                                    settings.stripMetadata ? "bg-accent" : "bg-border"
                                }`}
                            >
                                <div className={`absolute top-1 w-3 h-3 rounded-full bg-white transition-all ${settings.stripMetadata ? "left-[22px]" : "left-1"}`} />
                            </button>
                        </div>
                    </div>

                    <div className="h-[1px] bg-border" />

                    {/* Audio Section */}
                    <div className="space-y-6">
                        <h3 className="text-tiny font-bold uppercase tracking-widest text-text-muted">Audio</h3>

                        <div className="space-y-3">
                            <label className="text-[13px] font-medium text-text-secondary">Bitrate</label>
                            <div className="grid grid-cols-3 gap-2">
                                {audioBitrates.map((rate) => (
                                    <button
                                        key={rate}
                                        onClick={() => updateSetting("audioBitrate", rate)}
                                        className={`h-[32px] rounded-DEFAULT border text-tiny font-mono transition-all ${
                                            settings.audioBitrate === rate
                                            ? "bg-accent/10 border-accent text-accent"
                                            : "border-border text-text-muted hover:border-border-strong"
                                        }`}
                                    >
                                        {rate.toUpperCase()}
                                    </button>
                                ))}
                            </div>
                        </div>
                    </div>
                </div>

                <div className="p-6 border-t border-border bg-surface-raised flex gap-3">
                    <button onClick={onClose} className="btn-primary flex-1 h-[40px]">
                        Save changes
                    </button>
                </div>
            </div>
        </div>
    );
};
