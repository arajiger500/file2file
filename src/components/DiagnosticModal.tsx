import React, { useEffect } from "react";
import { X, AlertTriangle } from "lucide-react";
import { SidecarHealthReport, BinaryStatus } from "../types";

interface DiagnosticModalProps {
    isOpen: boolean;
    onClose: () => void;
    sidecars: SidecarHealthReport | null;
}

const BinaryRow = ({ status }: { status: BinaryStatus | undefined }) => {
    if (!status) return null;

    return (
        <div className="flex items-center justify-between p-3 bg-surface border border-border rounded-lg">
            <div className="flex items-center gap-3">
                <div className={`w-2 h-2 rounded-full ${status.available ? "bg-success" : "bg-error"}`} />
                <div>
                    <h4 className="text-[13px] font-semibold text-text-primary">{status.name}</h4>
                    <p className="text-tiny font-mono text-text-muted uppercase mt-0.5">
                        {status.absolute_path || status.path_or_sidecar}
                    </p>
                    {status.error && <p className="text-xs text-error mt-1">{status.error}</p>}
                </div>
            </div>
            <div className="text-right">
                <div className="text-[12px] font-mono text-text-secondary">
                    {status.version ? (
                        <span>{status.version.split(' ')[0]}</span>
                    ) : (
                        <span className="text-error font-semibold">{status.absolute_path ? "UNUSABLE" : "MISSING"}</span>
                    )}
                </div>
            </div>
        </div>
    );
};

export const DiagnosticModal: React.FC<DiagnosticModalProps> = ({ isOpen, onClose, sidecars }) => {
    useEffect(() => {
        if (!isOpen) return;
        const onKeyDown = (event: KeyboardEvent) => {
            if (event.key === "Escape") onClose();
        };
        window.addEventListener("keydown", onKeyDown);
        return () => window.removeEventListener("keydown", onKeyDown);
    }, [isOpen, onClose]);

    if (!isOpen) return null;
    const isBrowserFallback = sidecars?.ffmpeg?.path_or_sidecar === "browser";

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-6 bg-black/60 backdrop-blur-[2px]">
            <div role="dialog" aria-modal="true" aria-labelledby="diagnostics-title" className="relative w-full max-w-lg bg-surface border border-border rounded-xl shadow-2xl flex flex-col">
                <div className="h-[56px] px-6 border-b border-border bg-surface flex items-center justify-between">
                    <h2 id="diagnostics-title" className="text-[15px] font-semibold text-text-primary">Engine diagnostics</h2>
                    <button aria-label="Close diagnostics" onClick={onClose} className="p-2 -mr-2 text-text-muted hover:text-text-primary transition-colors">
                        <X className="w-5 h-5" />
                    </button>
                </div>

                <div className="p-6 space-y-6 flex-1 overflow-y-auto custom-scrollbar">
                    {!sidecars && (
                        <div role="alert" className="p-4 bg-error/10 border border-error/20 rounded-lg text-[12px] text-text-secondary">
                            Engine diagnostics are unavailable. Close this panel and use the header refresh control to try again.
                        </div>
                    )}
                    {isBrowserFallback && (
                        <div className="p-4 bg-accent/10 border border-accent/20 rounded-lg space-y-1">
                            <h3 className="text-[13px] font-semibold text-accent">Browser fallback mode</h3>
                            <p className="text-[12px] text-text-secondary leading-relaxed">
                                Lightweight image, PDF text, CSV, and JSON conversions run in this browser. Install the desktop app for native engines, archives, and media conversion.
                            </p>
                        </div>
                    )}
                    {sidecars && !sidecars.all_ready && !isBrowserFallback && (
                        <div className="p-4 bg-warning/10 border border-warning/20 rounded-lg flex gap-3">
                            <AlertTriangle className="w-4.5 h-4.5 text-warning shrink-0" />
                            <div className="space-y-1">
                                <h3 className="text-[13px] font-semibold text-warning">Limited engine availability</h3>
                                <p className="text-[12px] text-text-secondary leading-relaxed">
                                    Install or repair the listed engines, then use Refresh hardware info. Native data and archive conversions do not need these engines.
                                </p>
                            </div>
                        </div>
                    )}

                    {sidecars && !isBrowserFallback && (
                        <div className="space-y-2">
                            <BinaryRow status={sidecars.ffmpeg} />
                            <BinaryRow status={sidecars.ffprobe} />
                            <BinaryRow status={sidecars.imagemagick} />
                            <BinaryRow status={sidecars.pandoc} />
                            <BinaryRow status={sidecars.pdftotext} />
                            <BinaryRow status={sidecars.pdftohtml} />
                        </div>
                    )}
                </div>

                <div className="p-6 border-t border-border bg-surface-raised flex gap-3">
                    <button
                        onClick={onClose}
                        className="btn-secondary flex-1 h-[40px]"
                    >
                        Close
                    </button>
                </div>
            </div>
        </div>
    );
};
