import React from "react";
import { Cpu, Zap, Settings, Activity } from "lucide-react";
import appPackage from "../../package.json";
import { HardwareInfo, SidecarHealthReport } from "../types";

interface HeaderProps {
    hardware: HardwareInfo | null;
    sidecars: SidecarHealthReport | null;
    onToggleSettings: () => void;
    isSettingsOpen: boolean;
    onRefreshHardware: () => void;
    onOpenDiagnostics: () => void;
}

export const Header: React.FC<HeaderProps> = ({
    hardware,
    sidecars,
    onToggleSettings,
    isSettingsOpen,
    onRefreshHardware,
    onOpenDiagnostics,
}) => {
    const appVersion = appPackage.version || "0.1.0";

    return (
        <header className="h-[56px] border-b border-border bg-background px-4 flex items-center justify-between shrink-0">
            <div className="flex items-center gap-3">
                <span className="text-[18px] font-semibold text-text-primary tracking-tight">File2File</span>
                <span className="text-[12px] text-text-muted font-mono pt-0.5">v{appVersion}</span>
                <div className="h-4 w-[1px] bg-border mx-1" />
                <span className="text-[11px] text-text-muted uppercase tracking-wider font-medium pt-0.5">Local file conversion</span>
            </div>

            <div className="flex items-center gap-4">
                <button
                    onClick={onOpenDiagnostics}
                    className="flex items-center gap-2 text-[12px] text-text-secondary hover:text-text-primary transition-colors"
                    aria-label="Engine health status"
                >
                    <div className={`w-2 h-2 rounded-full ${sidecars?.all_ready ? "bg-success" : "bg-warning"}`} />
                    <span>Engines: {sidecars?.all_ready ? "Ready" : "Warning"}</span>
                </button>

                <button
                    onClick={onRefreshHardware}
                    className="flex items-center gap-2 text-[12px] text-text-secondary hover:text-text-primary transition-colors"
                    aria-label="Refresh hardware info"
                >
                    {hardware?.hardware_acceleration_supported ? (
                        <>
                            <Zap className="w-3.5 h-3.5 text-accent" />
                            <span>GPU: {hardware.gpu_vendor || "Accelerated"}</span>
                        </>
                    ) : (
                        <>
                            <Cpu className="w-3.5 h-3.5" />
                            <span>CPU: {hardware?.cpu_cores || 0} Cores</span>
                        </>
                    )}
                </button>

                <div className="flex items-center gap-1 ml-2">
                    <button
                        onClick={onOpenDiagnostics}
                        className="p-2 text-text-muted hover:text-text-primary hover:bg-surface-raised rounded-DEFAULT transition-all"
                        title="Diagnostics"
                    >
                        <Activity className="w-4 h-4" />
                    </button>
                    <button
                        onClick={onToggleSettings}
                        className={`p-2 rounded-DEFAULT transition-all ${
                            isSettingsOpen
                            ? "bg-accent/10 text-accent"
                            : "text-text-muted hover:text-text-primary hover:bg-surface-raised"
                        }`}
                        title="Settings"
                    >
                        <Settings className="w-4 h-4" />
                    </button>
                </div>
            </div>
        </header>
    );
};
