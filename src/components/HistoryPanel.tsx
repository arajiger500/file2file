import React from "react";
import { History, CheckCircle2, AlertCircle, Clock, HardDrive, Trash2 } from "lucide-react";
import { ConversionResult } from "../types";

interface HistoryPanelProps {
    history: ConversionResult[];
    onClearHistory: () => void;
}

export const HistoryPanel: React.FC<HistoryPanelProps> = ({
    history,
    onClearHistory,
}) => {
    const formatBytes = (bytes: number): string => {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
    };

    const calculateRatio = (orig: number, conv: number) => {
        if (orig === 0 || conv === 0) return null;
        const diff = orig - conv;
        const percent = Math.round((diff / orig) * 100);
        return percent;
    };

    return (
        <div className="bg-surface rounded-2xl border border-border p-5 space-y-4">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <History className="w-4 h-4 text-brand-400" />
                    <h3 className="font-semibold text-sm text-slate-200">Conversion History & Logs</h3>
                </div>
                {history.length > 0 && (
                    <button
                        onClick={onClearHistory}
                        className="text-xs text-slate-400 hover:text-rose-400 flex items-center gap-1 transition-colors"
                    >
                        <Trash2 className="w-3.5 h-3.5" />
                        <span>Clear Logs</span>
                    </button>
                )}
            </div>

            {history.length === 0 ? (
                <p className="text-xs text-slate-500 text-center py-4">
                    No conversions recorded yet in this session.
                </p>
            ) : (
                <div className="space-y-2.5 max-h-60 overflow-y-auto pr-1 text-xs">
                    {history.map((item) => {
                        const savings = calculateRatio(item.original_size_bytes, item.converted_size_bytes);
                        return (
                            <div
                                key={item.job_id}
                                className="p-3 rounded-xl bg-surface-raised border border-border/80 space-y-1.5"
                            >
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center gap-2 truncate max-w-[70%]">
                                        {item.success ? (
                                            <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
                                        ) : (
                                            <AlertCircle className="w-4 h-4 text-rose-400 shrink-0" />
                                        )}
                                        <span className="font-medium text-slate-200 truncate">
                                            {item.output_path.split("/").pop() || item.output_path}
                                        </span>
                                    </div>

                                    <div className="flex items-center gap-2">
                                        {item.success && savings !== null && (
                                            <span
                                                className={`text-[10px] font-semibold px-1.5 py-0.5 rounded border ${savings > 0
                                                        ? "bg-emerald-950/40 border-emerald-500/30 text-emerald-300"
                                                        : "bg-surface border-border text-slate-400"
                                                    }`}
                                            >
                                                {savings > 0 ? `-${savings}% smaller` : `${Math.abs(savings)}% larger`}
                                            </span>
                                        )}
                                        <span className="text-[10px] text-slate-400 flex items-center gap-1 font-mono">
                                            <Clock className="w-3 h-3" />
                                            {item.elapsed_ms}ms
                                        </span>
                                    </div>
                                </div>

                                <div className="flex items-center justify-between text-[11px] text-slate-400 pt-1 border-t border-border/40">
                                    <div className="flex items-center gap-2 font-mono">
                                        <HardDrive className="w-3 h-3 text-slate-500" />
                                        <span>{formatBytes(item.original_size_bytes)}</span>
                                        <span>→</span>
                                        <span className="text-slate-200">{formatBytes(item.converted_size_bytes)}</span>
                                    </div>
                                    <span className="truncate max-w-[200px] text-slate-500" title={item.output_path}>
                                        {item.output_path}
                                    </span>
                                </div>

                                {item.error && (
                                    <p className="text-[11px] text-rose-300 bg-rose-950/30 border border-rose-500/20 rounded p-1.5 mt-1 font-mono">
                                        {item.error}
                                    </p>
                                )}
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
};
