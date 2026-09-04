import React from "react";
import { X, AlertTriangle, CheckCircle2, Info, ArrowRight } from "lucide-react";
import { SidecarHealthReport, BinaryStatus } from "../types";

interface DiagnosticModalProps {
    isOpen: boolean;
    onClose: () => void;
    sidecars: SidecarHealthReport | null;
}

const BinaryRow = ({ status }: { status: BinaryStatus | undefined }) => {
    if (!status) return null;

    const isMissing = !status.available;
    const isBundled = status.path_or_sidecar === "bundled";

    return (
        <div className="flex items-center justify-between p-4 rounded-2xl bg-zinc-900/50 border border-zinc-800">
            <div className="flex items-center gap-4">
                <div className={`p-2 rounded-lg ${isMissing ? "bg-red-500/10" : "bg-emerald-500/10"}`}>
                    {isMissing ? (
                        <AlertTriangle className="w-4 h-4 text-red-400" />
                    ) : (
                        <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                    )}
                </div>
                <div>
                    <h4 className="text-sm font-bold text-white uppercase tracking-wider">{status.name}</h4>
                    <p className="text-[10px] text-zinc-500 mt-0.5">
                        {isMissing ? "Missing or inaccessible" : `Found via ${status.path_or_sidecar}`}
                    </p>
                </div>
            </div>
            <div className="text-right text-[10px] font-mono">
                {status.version ? (
                    <span className="text-zinc-400 truncate max-w-[150px] inline-block">{status.version.split(' ')[0]}</span>
                ) : (
                    <span className="text-red-500 font-bold uppercase">Critical</span>
                )}
            </div>
        </div>
    );
};

export const DiagnosticModal: React.FC<DiagnosticModalProps> = ({ isOpen, onClose, sidecars }) => {
    if (!isOpen || !sidecars) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-6 bg-black/60 backdrop-blur-sm">
            <div className="relative w-full max-w-xl bg-zinc-950 border border-zinc-800 rounded-[2.5rem] shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-300">
                <div className="p-8 border-b border-zinc-900 bg-gradient-to-b from-zinc-900/50 to-transparent">
                    <div className="flex items-center justify-between">
                        <div className="space-y-1">
                            <h2 className="text-2xl font-black text-white uppercase tracking-tighter">Engine Health</h2>
                            <p className="text-zinc-500 text-xs">Self-diagnostic report for conversion cores</p>
                        </div>
                        <button onClick={onClose} className="p-2 rounded-xl bg-zinc-900 border border-zinc-800 text-zinc-500 hover:text-white transition-all">
                            <X className="w-5 h-5" />
                        </button>
                    </div>
                </div>

                <div className="p-8 space-y-6">
                    {!sidecars.all_ready && (
                        <div className="p-4 rounded-2xl bg-amber-500/10 border border-amber-500/20 flex gap-4">
                            <Info className="w-5 h-5 text-amber-500 shrink-0" />
                            <div className="space-y-1">
                                <h3 className="text-sm font-bold text-amber-200">Limited Capability Detected</h3>
                                <p className="text-[11px] text-amber-200/60 leading-relaxed">
                                    Some conversion engines are missing. File2File will try to use alternatives from your system, but performance and format support might be limited.
                                </p>
                            </div>
                        </div>
                    )}

                    <div className="space-y-3">
                        <BinaryRow status={sidecars.ffmpeg} />
                        <BinaryRow status={sidecars.ffprobe} />
                        <BinaryRow status={sidecars.imagemagick} />
                        <BinaryRow status={sidecars.pandoc} />
                        <BinaryRow status={sidecars.pdftotext} />
                    </div>

                    <div className="pt-4 flex flex-col gap-3">
                        <button
                            onClick={onClose}
                            className="w-full py-4 rounded-2xl bg-white text-black font-black text-xs uppercase tracking-[0.2em] hover:bg-zinc-200 transition-all shadow-xl active:scale-95"
                        >
                            Return to Dashboard
                        </button>
                        <p className="text-[9px] text-center text-zinc-600 uppercase font-bold tracking-widest flex items-center justify-center gap-2">
                            Manual Setup Guide <ArrowRight className="w-3 h-3" />
                        </p>
                    </div>
                </div>
            </div>
        </div>
    );
};
