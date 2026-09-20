import { AlertTriangle } from "lucide-react";
import { QuickPreset, SidecarHealthReport } from "../types";

interface QuickConvertersProps {
    presets: QuickPreset[];
    onSelectPreset: (preset: QuickPreset) => void;
    sidecars: SidecarHealthReport | null;
}

export const QuickConverters: React.FC<QuickConvertersProps> = ({
    presets,
    onSelectPreset,
    sidecars,
}) => {
    const isEngineMissing = (preset: QuickPreset) => {
        if (!sidecars) return false;
        if (preset.from_category === "document" && (!sidecars.pandoc.available || !sidecars.pdftohtml.available)) return true;
        if (preset.from_category === "image" && !sidecars.imagemagick.available) return true;
        if (preset.from_category === "video" && !sidecars.ffmpeg.available) return true;
        return false;
    };
    return (
        <div className="space-y-4">
            <h3 className="text-[14px] font-semibold text-text-primary">Quick convert</h3>

            <div className="flex flex-wrap gap-2">
                {presets.map((preset) => {
                    const missing = isEngineMissing(preset);
                    return (
                        <button
                            key={preset.id}
                            onClick={() => onSelectPreset(preset)}
                            className={`btn-secondary h-[40px] px-4 flex items-center gap-2 group border-border hover:border-accent ${
                                missing ? "opacity-70 grayscale-[0.5]" : ""
                            }`}
                        >
                            <span className="text-[13px] font-medium text-text-primary group-hover:text-accent">
                                {preset.title.replace(/_/g, " ")}
                            </span>
                            <div className="flex items-center gap-1.5">
                                <span className="text-tiny text-text-muted font-mono uppercase pt-0.5">
                                    {preset.to_format}
                                </span>
                                {missing && (
                                    <AlertTriangle className="w-3 h-3 text-warning"  />
                                )}
                            </div>
                        </button>
                    );
                })}
            </div>
        </div>
    );
};
