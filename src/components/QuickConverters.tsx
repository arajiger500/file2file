import React from "react";
import { QuickPreset } from "../types";

interface QuickConvertersProps {
    presets: QuickPreset[];
    onSelectPreset: (preset: QuickPreset) => void;
}

export const QuickConverters: React.FC<QuickConvertersProps> = ({
    presets,
    onSelectPreset,
}) => {
    return (
        <div className="space-y-4">
            <h3 className="text-[14px] font-semibold text-text-primary">Quick convert</h3>

            <div className="flex flex-wrap gap-2">
                {presets.map((preset) => (
                    <button
                        key={preset.id}
                        onClick={() => onSelectPreset(preset)}
                        className="btn-secondary h-[40px] px-4 flex items-center gap-2 group border-border hover:border-accent"
                    >
                        <span className="text-[13px] font-medium text-text-primary group-hover:text-accent">
                            {preset.title.replace(/_/g, " ")}
                        </span>
                        <span className="text-tiny text-text-muted font-mono uppercase pt-0.5">
                            {preset.to_format}
                        </span>
                    </button>
                ))}
            </div>
        </div>
    );
};
