import React from "react";
import { Cpu, Zap, Sliders, ShieldCheck, RefreshCw } from "lucide-react";
import appPackage from "../../package.json";
import { HardwareInfo, SidecarHealthReport } from "../types";

interface HeaderProps {
    hardware: HardwareInfo | null;
    sidecars: SidecarHealthReport | null;
    onToggleSettings: () => void;
    isSettingsOpen: boolean;
    onRefreshHardware: () => void;
}

export const Header: React.FC<HeaderProps> = ({
    hardware,
    sidecars,
    onToggleSettings,
    isSettingsOpen,
    onRefreshHardware,
}) => {
    const appVersion = appPackage.version || "0.1.0";

    return (
        <header className="border-b border-border bg-surface/80 backdrop-blur px-6 py-3.5 flex items-center justify-between sticky top-0 z-30">
            {/* Brand logo & tagline */}
            <div className="flex items-center gap-3">
                <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-brand-500 to-blue-700 flex items-center justify-center shadow-lg shadow-brand-500/20">
                    <RefreshCw className="w-5 h-5 text-white animate-spin-slow" />
                </div>
                <div>
                    <div className="flex items-center gap-2">
                        <h1 className="font-bold text-lg text-slate-100 tracking-tight">File2File</h1>
                        <span className="text-[10px] uppercase font-semibold tracking-wider bg-brand-500/20 text-brand-400 border border-brand-500/30 px-1.5 py-0.5 rounded">
                            v{appVersion}
                        </span>
                    </div>
                    <p className="text-xs text-slate-400">100% Offline & Private File Converter</p>
                </div>
            </div>

            {/* Hardware status indicators & controls */}
            <div className="flex items-center gap-3">
                {/* Sidecar status pill */}
                <div
                    className="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs bg-surface-raised border border-border text-slate-300"
                    title="Conversion tools: FFmpeg, Pandoc, ImageMagick, and Poppler"
                >
                    <ShieldCheck className={`w-3.5 h-3.5 ${sidecars?.all_ready ? "text-emerald-400" : "text-amber-400"}`} />
                    <span className="font-medium">Tools: {sidecars?.all_ready ? "Active" : "Needs setup"}</span>
                </div>

                {/* Hardware acceleration badge */}
                <button
                    onClick={onRefreshHardware}
                    className={`flex items-center gap-2 px-3 py-1 rounded-full text-xs font-medium border transition-colors ${hardware?.hardware_acceleration_supported
                        ? "bg-emerald-950/40 border-emerald-500/30 text-emerald-300 hover:bg-emerald-950/60"
                        : "bg-surface-raised border-border text-slate-300 hover:bg-surface"
                        }`}
                    title={`GPU: ${hardware?.gpu_vendor || "None"} | CPU Cores: ${hardware?.cpu_cores || 0}`}
                >
                    {hardware?.hardware_acceleration_supported ? (
                        <>
                            <Zap className="w-3.5 h-3.5 text-emerald-400 fill-emerald-400/30" />
                            <span>GPU: {hardware.gpu_vendor || "Hardware"} Accel</span>
                        </>
                    ) : (
                        <>
                            <Cpu className="w-3.5 h-3.5 text-slate-400" />
                            <span>CPU ({hardware?.cpu_cores || 4} Threads)</span>
                        </>
                    )}
                </button>

                {/* Settings toggle button */}
                <button
                    onClick={onToggleSettings}
                    className={`p-2 rounded-lg border transition-all ${isSettingsOpen
                        ? "bg-brand-500/20 border-brand-500 text-brand-400 shadow-sm"
                        : "bg-surface-raised border-border text-slate-300 hover:text-white hover:border-slate-600"
                        }`}
                    title="Advanced Conversion Settings"
                >
                    <Sliders className="w-4 h-4" />
                </button>
            </div>
        </header>
    );
};
