import { useState, useMemo } from "react";
import { Search, ArrowLeft, AlertTriangle } from "lucide-react";
import { FileItem, FormatOption, SidecarHealthReport } from "../types";

interface FormatSelectionPageProps {
    files: FileItem[];
    availableFormats: FormatOption[];
    onSelectFormat: (format: FormatOption) => void;
    onBack: () => void;
    sidecars: SidecarHealthReport | null;
}

export const FormatSelectionPage: React.FC<FormatSelectionPageProps> = ({ availableFormats, onSelectFormat, onBack, sidecars }) => {
    const [search, setSearch] = useState("");
    const [filter, setFilter] = useState("All");

    const isEngineMissing = (engine: string) => {
        if (!sidecars) return false;
        const e = engine.toLowerCase();
        if (e.includes("ffmpeg") && !sidecars.ffmpeg.available) return true;
        if (e.includes("imagemagick") && !sidecars.imagemagick.available) return true;
        if (e.includes("pandoc") && !sidecars.pandoc.available) return true;
        if (e.includes("poppler") && (!sidecars.pdftotext.available || !sidecars.pdftohtml.available)) return true;
        return false;
    };

    const categories = useMemo(() => ["All", ...Array.from(new Set(availableFormats.map(f => f.subcategory)))], [availableFormats]);

    const filtered = availableFormats.filter(f =>
        (filter === "All" || f.subcategory === filter) &&
        (f.name.toLowerCase().includes(search.toLowerCase()) || f.extension.toLowerCase().includes(search.toLowerCase()))
    );

    return (
        <div className="h-full flex flex-col gap-6">
            {/* Action Bar */}
            <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
                <div className="flex items-center gap-4">
                    <button onClick={onBack} className="p-2 text-text-muted hover:text-text-primary hover:bg-surface-raised rounded-DEFAULT transition-all">
                        <ArrowLeft className="w-5 h-5" />
                    </button>
                    <div>
                        <h2 className="text-[24px] font-semibold text-text-primary tracking-tight">Choose output format</h2>
                        <p className="text-[13px] text-text-secondary">{filtered.length} formats available</p>
                    </div>
                </div>

                <div className="relative w-full md:w-72">
                    <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-text-muted" />
                    <input
                        type="text"
                        placeholder="Search formats..."
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                        className="input-field w-full pl-9"
                    />
                </div>
            </div>

            {/* Filter Tabs */}
            <div className="flex items-center gap-1 border-b border-border">
                {categories.map((c: string) => (
                    <button
                        key={c}
                        onClick={() => setFilter(c)}
                        className={`px-4 py-2.5 text-[13px] font-medium transition-all relative ${
                            filter === c
                            ? "text-accent after:absolute after:bottom-0 after:left-0 after:right-0 after:h-0.5 after:bg-accent"
                            : "text-text-secondary hover:text-text-primary"
                        }`}
                    >
                        {c}
                    </button>
                ))}
            </div>

            {/* Format Grid */}
            <div className="flex-1 overflow-y-auto custom-scrollbar pr-2">
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3 pb-10">
                    {filtered.map(f => (
                        <button
                            key={f.extension}
                            onClick={() => onSelectFormat(f)}
                            className="group p-4 bg-surface border border-border rounded-lg hover:border-accent hover:bg-surface-raised text-left transition-all"
                        >
                            <div className="flex items-center justify-between mb-3">
                                <span className="text-[16px] font-mono font-bold text-text-primary group-hover:text-accent">
                                    .{f.extension}
                                </span>
                                {f.is_lossless && (
                                    <span className="px-1.5 py-0.5 bg-success/10 text-success text-[10px] font-semibold rounded-sm uppercase tracking-tighter">Lossless</span>
                                )}
                            </div>

                            <div className="space-y-1">
                                <h3 className="text-[13px] font-semibold text-text-primary truncate">{f.name}</h3>
                                <p className="text-[11px] text-text-muted leading-relaxed line-clamp-2 h-8">{f.description}</p>
                            </div>

                            <div className="mt-4 pt-3 border-t border-border flex items-center justify-between">
                                <div className="flex items-center gap-1.5">
                                    <span className="text-tiny font-mono text-text-disabled uppercase">{f.sidecar_engine}</span>
                                    {isEngineMissing(f.sidecar_engine) && (
                                        <AlertTriangle className="w-3 h-3 text-warning"  />
                                    )}
                                </div>
                                {f.is_recommended && (
                                    <span className="text-tiny text-accent font-medium uppercase tracking-widest">Recommended</span>
                                )}
                            </div>
                        </button>
                    ))}
                </div>
            </div>
        </div>
    );
};
