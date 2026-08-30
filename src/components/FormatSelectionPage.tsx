import React, { useState, useMemo } from "react";
import { Search, ArrowLeft, Zap, Sparkles, Filter, Grid, List } from "lucide-react";
import { FileItem, FormatOption } from "../types";

interface FormatSelectionPageProps {
    files: FileItem[];
    availableFormats: FormatOption[];
    onSelectFormat: (format: FormatOption) => void;
    onBack: () => void;
}

export const FormatSelectionPage: React.FC<FormatSelectionPageProps> = ({ files, availableFormats, onSelectFormat, onBack }) => {
    const [search, setSearch] = useState("");
    const [filter, setFilter] = useState("All");

    const subcategories = useMemo(() => ["All", ...Array.from(new Set(availableFormats.map(f => f.subcategory)))], [availableFormats]);

    const filtered = availableFormats.filter(f =>
        (filter === "All" || f.subcategory === filter) &&
        (f.name.toLowerCase().includes(search.toLowerCase()) || f.extension.toLowerCase().includes(search.toLowerCase()))
    );

    return (
        <div className="space-y-8 pb-20 max-w-6xl mx-auto">
            <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-6">
                <div className="flex items-center gap-4">
                    <button onClick={onBack} className="p-3 rounded-2xl bg-zinc-900 border border-zinc-800 hover:border-zinc-600 transition-all text-zinc-400 hover:text-white">
                        <ArrowLeft className="w-5 h-5" />
                    </button>
                    <div>
                        <h2 className="text-2xl font-bold text-white tracking-tight">Select Export Format</h2>
                        <p className="text-zinc-500 text-sm">{files.length} items ready for processing</p>
                    </div>
                </div>

                <div className="relative w-full md:w-80 group">
                    <Search className="w-4 h-4 absolute left-4 top-1/2 -translate-y-1/2 text-zinc-500 group-focus-within:text-blue-400 transition-colors" />
                    <input
                        type="text"
                        placeholder="Search 100+ formats..."
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                        className="w-full bg-zinc-900/50 border border-zinc-800 rounded-2xl pl-11 pr-4 py-3 text-sm focus:outline-none focus:border-blue-500/50 focus:bg-zinc-900 transition-all"
                    />
                </div>
            </div>

            <div className="flex items-center gap-2 overflow-x-auto pb-2 no-scrollbar">
                {subcategories.map(s => (
                    <button
                        key={s}
                        onClick={() => setFilter(s)}
                        className={`px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider transition-all whitespace-nowrap border ${
                            filter === s ? "bg-blue-600 border-blue-500 text-white shadow-lg shadow-blue-500/20" : "bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300 hover:border-zinc-700"
                        }`}
                    >
                        {s}
                    </button>
                ))}
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                {filtered.map(f => (
                    <div
                        key={f.extension}
                        onClick={() => onSelectFormat(f)}
                        className="group p-5 rounded-3xl bg-zinc-900/40 border border-zinc-800 hover:border-blue-500/50 hover:bg-zinc-900 transition-all duration-300 cursor-pointer relative overflow-hidden"
                    >
                        <div className="absolute top-0 right-0 w-24 h-24 bg-blue-500/5 blur-3xl group-hover:bg-blue-500/10 transition-colors" />

                        <div className="relative z-10 flex items-center justify-between mb-4">
                            <span className="font-mono text-lg font-black text-white group-hover:text-blue-400 transition-colors uppercase">.{f.extension}</span>
                            {f.is_recommended && (
                                <div className="p-1.5 rounded-lg bg-amber-500/10 border border-amber-500/20">
                                    <Sparkles className="w-3 h-3 text-amber-500" />
                                </div>
                            )}
                        </div>

                        <div className="relative z-10 space-y-2">
                            <h3 className="text-sm font-bold text-zinc-200">{f.name}</h3>
                            <p className="text-[11px] text-zinc-500 leading-relaxed line-clamp-2">{f.description}</p>
                        </div>

                        <div className="mt-6 pt-4 border-t border-zinc-800/50 flex items-center justify-between">
                            <span className="text-[9px] font-bold text-zinc-600 uppercase tracking-tighter">{f.sidecar_engine} Core</span>
                            <div className="flex items-center gap-1.5">
                                <div className={`w-1.5 h-1.5 rounded-full ${f.is_lossless ? "bg-emerald-500" : "bg-zinc-700"}`} />
                                <span className="text-[9px] font-bold text-zinc-600 uppercase">{f.is_lossless ? "Lossless" : "Standard"}</span>
                            </div>
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
};
