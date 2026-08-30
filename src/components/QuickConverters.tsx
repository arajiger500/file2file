import React from "react";
import { Sparkles, Image, Video, Film, Music, FileText } from "lucide-react";
import { QuickPreset } from "../types";

interface QuickConvertersProps {
    presets: QuickPreset[];
    onSelectPreset: (preset: QuickPreset) => void;
}

export const QuickConverters: React.FC<QuickConvertersProps> = ({
    presets,
    onSelectPreset,
}) => {
    const getIcon = (iconName: string) => {
        switch (iconName) {
            case "image":
                return <Image className="w-4 h-4 text-emerald-400" />;
            case "video":
                return <Video className="w-4 h-4 text-blue-400" />;
            case "film":
                return <Film className="w-4 h-4 text-purple-400" />;
            case "music":
                return <Music className="w-4 h-4 text-pink-400" />;
            case "file-text":
                return <FileText className="w-4 h-4 text-amber-400" />;
            default:
                return <Sparkles className="w-4 h-4 text-brand-400" />;
        }
    };

    return (
        <div className="space-y-3">
            <div className="flex items-center gap-2">
                <Sparkles className="w-4 h-4 text-amber-400" />
                <h3 className="font-semibold text-xs text-slate-300 uppercase tracking-wider">
                    Quick Workflows
                </h3>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2.5">
                {presets.map((preset) => (
                    <button
                        key={preset.id}
                        onClick={() => onSelectPreset(preset)}
                        className="flex flex-col text-left p-3 rounded-xl bg-surface border border-border/80 hover:border-brand-500/50 hover:bg-surface-raised transition-all group"
                    >
                        <div className="flex items-center justify-between w-full mb-1">
                            <div className="flex items-center gap-2">
                                <div className="p-1.5 rounded-lg bg-surface-raised border border-border">
                                    {getIcon(preset.icon)}
                                </div>
                                <div>
                                    <span className="font-semibold text-xs text-slate-200 group-hover:text-brand-300 transition-colors block">
                                        {preset.title}
                                    </span>
                                </div>
                            </div>
                            <span className="text-[9px] uppercase font-bold px-1.5 py-0.5 rounded bg-brand-500/20 text-brand-300 border border-brand-500/30">
                                {preset.badge || "Quick"}
                            </span>
                        </div>
                        <p className="text-[11px] text-slate-400 line-clamp-1">{preset.description}</p>
                    </button>
                ))}
            </div>
        </div>
    );
};
