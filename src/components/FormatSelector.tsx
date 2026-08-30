import React from "react";
import { Info, Sparkles, Check, FileCheck, Layers } from "lucide-react";
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
            <div className="bg-surface rounded-2xl border border-border p-6 text-center text-slate-400 text-xs">
                Add a file above to view compatible output formats and conversion options.
            </div>
        );
    }

    return (
        <div className="bg-surface rounded-2xl border border-border p-5 space-y-4">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <Layers className="w-4 h-4 text-brand-400" />
                    <h3 className="font-semibold text-sm text-slate-200">
                        Target Format for <span className="uppercase text-brand-400 font-mono">.{inputExtension}</span>
                    </h3>
                </div>
                <span className="text-xs text-slate-400">
                    {availableFormats.length} supported targets
                </span>
            </div>

            {/* Grid of format cards */}
            <div className="grid grid-cols-2 sm:grid-cols-3 gap-2.5">
                {availableFormats.map((fmt) => {
                    const isSelected = fmt.extension === selectedFormat;
                    return (
                        <button
                            key={fmt.extension}
                            onClick={() => onSelectFormat(fmt.extension)}
                            className={`text-left p-3 rounded-xl border transition-all relative ${isSelected
                                    ? "bg-brand-500/10 border-brand-500 text-white shadow-md shadow-brand-500/10 ring-1 ring-brand-500/50"
                                    : "bg-surface-raised border-border/70 hover:border-slate-500 text-slate-300 hover:text-white"
                                }`}
                        >
                            <div className="flex items-center justify-between mb-1">
                                <span className="font-bold text-sm uppercase tracking-wide font-mono">
                                    .{fmt.extension}
                                </span>
                                {isSelected ? (
                                    <div className="w-4 h-4 rounded-full bg-brand-500 flex items-center justify-center">
                                        <Check className="w-2.5 h-2.5 text-white stroke-[3]" />
                                    </div>
                                ) : (
                                    <span
                                        className={`text-[9px] uppercase font-semibold px-1.5 py-0.2 rounded border ${fmt.is_lossless
                                                ? "bg-emerald-950/50 border-emerald-500/30 text-emerald-300"
                                                : "bg-amber-950/40 border-amber-500/30 text-amber-300"
                                            }`}
                                    >
                                        {fmt.is_lossless ? "Lossless" : "Lossy"}
                                    </span>
                                )}
                            </div>
                            <p className="text-xs text-slate-400 truncate">{fmt.name}</p>
                        </button>
                    );
                })}
            </div>

            {/* Contextual Description & Comparison Card */}
            {activeFormat && (
                <div className="bg-surface-raised rounded-xl border border-border/80 p-3.5 text-xs space-y-2">
                    <div className="flex items-start gap-2.5">
                        <Info className="w-4 h-4 text-brand-400 shrink-0 mt-0.5" />
                        <div className="space-y-1">
                            <div className="flex items-center gap-2">
                                <span className="font-semibold text-slate-200">{activeFormat.name}</span>
                                <span className="text-[10px] text-slate-400 bg-surface px-1.5 py-0.5 rounded border border-border font-mono">
                                    engine: {activeFormat.sidecar_engine}
                                </span>
                            </div>
                            <p className="text-slate-300 leading-relaxed">{activeFormat.description}</p>
                        </div>
                    </div>

                    {/* Comparison Note Callout (e.g. WEBP vs PNG) */}
                    {activeFormat.comparison_note && (
                        <div className="flex items-start gap-2 bg-brand-500/10 border border-brand-500/30 rounded-lg p-2.5 text-brand-200">
                            <Sparkles className="w-3.5 h-3.5 text-brand-400 shrink-0 mt-0.5" />
                            <div className="space-y-0.5">
                                <span className="font-semibold text-[11px] uppercase tracking-wider text-brand-300">
                                    Format Comparison Note
                                </span>
                                <p className="text-xs text-slate-300 leading-normal">
                                    {activeFormat.comparison_note}
                                </p>
                            </div>
                        </div>
                    )}

                    {/* Recommended for tags */}
                    {activeFormat.recommended_for.length > 0 && (
                        <div className="flex items-center gap-1.5 flex-wrap pt-1">
                            <span className="text-slate-500 text-[11px]">Best for:</span>
                            {activeFormat.recommended_for.map((tag) => (
                                <span
                                    key={tag}
                                    className="px-2 py-0.5 rounded-md bg-surface border border-border text-slate-300 text-[11px] flex items-center gap-1"
                                >
                                    <FileCheck className="w-2.5 h-2.5 text-brand-400" />
                                    {tag}
                                </span>
                            ))}
                        </div>
                    )}
                </div>
            )}
        </div>
    );
};
