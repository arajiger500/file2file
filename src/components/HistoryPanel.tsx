import React from "react";
import { History, CheckCircle2, AlertCircle } from "lucide-react";
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
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + sizes[i];
    };

    return (
        <div className="flex flex-col h-full gap-4">
            <div className="flex items-center justify-between">
                <h3 className="text-tiny uppercase tracking-widest text-text-muted font-bold">Recent conversions</h3>
                {history.length > 0 && (
                    <button
                        onClick={onClearHistory}
                        className="text-tiny font-medium text-text-muted hover:text-text-primary transition-colors"
                    >
                        Clear history
                    </button>
                )}
            </div>

            <div className="flex-1 overflow-y-auto custom-scrollbar -mx-1 px-1">
                {history.length === 0 ? (
                    <div className="h-full flex flex-col items-center justify-center text-center p-6 space-y-3 opacity-30">
                        <History className="w-8 h-8 text-text-muted" />
                        <span className="text-[12px] text-text-muted">No recent activity</span>
                    </div>
                ) : (
                    <div className="space-y-1.5">
                        {history.map((item) => (
                            <div
                                key={item.job_id}
                                className="p-3 bg-surface-raised border border-border rounded-lg space-y-2 group"
                            >
                                <div className="flex items-center justify-between gap-2">
                                    <div className="flex items-center gap-2 truncate">
                                        {item.success ? (
                                            <CheckCircle2 className="w-3.5 h-3.5 text-success shrink-0" />
                                        ) : (
                                            <AlertCircle className="w-3.5 h-3.5 text-error shrink-0" />
                                        )}
                                        <span className="text-[12px] font-medium text-text-secondary truncate">
                                            {item.output_path.split(/[\\/]/).pop() || item.output_path}
                                        </span>
                                    </div>
                                    <span className="text-tiny font-mono text-text-disabled shrink-0">{item.elapsed_ms}ms</span>
                                </div>

                                <div className="flex items-center justify-between text-tiny font-mono text-text-muted uppercase">
                                    <div className="flex items-center gap-1.5">
                                        <span>{formatBytes(item.original_size_bytes)}</span>
                                        <span className="text-text-disabled">→</span>
                                        <span className={item.success ? "text-text-secondary" : "text-text-muted"}>
                                            {formatBytes(item.converted_size_bytes)}
                                        </span>
                                    </div>
                                    {item.success && (
                                        <span className="text-text-disabled">
                                            {item.output_path.split('.').pop()?.toUpperCase()}
                                        </span>
                                    )}
                                </div>
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
};
