import React from "react";
import { Info, Check } from "lucide-react";
import { FormatOption } from "../types";

interface FormatSelectorProps {
    availableFormats: FormatOption[];
    selectedFormat: string;
    onSelectFormat: (format: string) => void;
    inputExtension: string;
}

export const FormatSelector: React.FC<FormatSelectorProps> = ({
    availableFormats,
    selectedFormat,
    onSelectFormat,
    inputExtension,
}) => {
    const activeFormat = availableFormats.find((f) => f.extension === selectedFormat);

    if (availableFormats.length === 0) {
        return (
            <div className="bg-surface rounded-lg border border-border p-6 text-center text-text-muted text-[13px]">
                Add a file to view compatible output formats.
            </div>
        );
    }

    return (
        <div className="bg-surface rounded-lg border border-border p-5 space-y-6">
            <div className="flex items-center justify-between">
                <h3 className="text-[14px] font-semibold text-text-primary">
                    Output format for <span className="font-mono text-accent uppercase">.{inputExtension}</span>
                </h3>
                <span className="text-tiny font-medium text-text-muted uppercase tracking-widest">
                    {availableFormats.length} formats available
                </span>
            </div>

            {/* Grid of format cards */}
            <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                {availableFormats.map((fmt) => {
                    const isSelected = fmt.extension === selectedFormat;
                    return (
                        <button
                            key={fmt.extension}
                            onClick={() => onSelectFormat(fmt.extension)}
                            className={`text-left p-3 rounded-lg border transition-all relative ${
                                isSelected
                                ? "bg-accent/5 border-accent text-text-primary"
                                : "bg-surface-raised border-border hover:border-border-strong text-text-secondary hover:text-text-primary"
                            }`}
                        >
                            <div className="flex items-center justify-between mb-1">
                                <span className="font-bold text-[14px] uppercase font-mono tracking-tight">
                                    .{fmt.extension}
                                </span>
                                {isSelected && (
                                    <div className="w-4 h-4 rounded-full bg-accent flex items-center justify-center">
                                        <Check className="w-2.5 h-2.5 text-white" />
                                    </div>
                                )}
                            </div>
                            <p className="text-tiny text-text-muted truncate">{fmt.name}</p>
                        </button>
                    );
                })}
            </div>

            {/* Contextual Description */}
            {activeFormat && (
                <div className="bg-surface-raised rounded-lg border border-border p-4 space-y-3">
                    <div className="flex items-start gap-3">
                        <Info className="w-4 h-4 text-accent shrink-0 mt-0.5" />
                        <div className="space-y-1">
                            <div className="flex items-center gap-2">
                                <span className="text-[13px] font-semibold text-text-primary">{activeFormat.name}</span>
                                <span className="text-tiny font-mono text-text-disabled uppercase px-1.5 py-0.5 border border-border rounded-sm">
                                    {activeFormat.sidecar_engine}
                                </span>
                            </div>
                            <p className="text-[12px] text-text-secondary leading-relaxed">{activeFormat.description}</p>
                        </div>
                    </div>

                    {activeFormat.comparison_note && (
                        <div className="bg-background/50 border border-border rounded-md p-3">
                            <p className="text-[12px] text-text-secondary leading-normal">
                                <span className="font-semibold text-text-primary">Note: </span>
                                {activeFormat.comparison_note}
                            </p>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
};
