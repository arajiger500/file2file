import { isTauri } from "./services/api";
import { useEffect, useState } from "react";
import { Header } from "./components/Header";
import { DropZone } from "./components/DropZone";
import { FormatSelectionPage } from "./components/FormatSelectionPage";
import { ConversionStudio } from "./components/ConversionStudio";
import { AdvancedSettingsDrawer } from "./components/AdvancedSettingsDrawer";
import { QuickConverters } from "./components/QuickConverters";
import { HistoryPanel } from "./components/HistoryPanel";
import { DiagnosticModal } from "./components/DiagnosticModal";
import { api } from "./services/api";
import {
    FileItem,
    FormatOption,
    HardwareInfo,
    SidecarHealthReport,
    QuickPreset,
    AdvancedSettings,
    ConversionResult,
} from "./types";
import { Shield, Database, ArrowRight } from "lucide-react";

type AppStep = "upload" | "select-format" | "convert-box";

export function App() {
    const [currentStep, setCurrentStep] = useState<AppStep>("upload");
    const [files, setFiles] = useState<FileItem[]>([]);
    const [hardware, setHardware] = useState<HardwareInfo | null>(null);
    const [sidecars, setSidecars] = useState<SidecarHealthReport | null>(null);
    const [presets, setPresets] = useState<QuickPreset[]>([]);
    const [availableFormats, setAvailableFormats] = useState<FormatOption[]>([]);
    const [selectedFormatOption, setSelectedFormatOption] = useState<FormatOption | null>(null);
    const [isSettingsOpen, setIsSettingsOpen] = useState<boolean>(false);
    const [isDiagnosticOpen, setIsDiagnosticOpen] = useState<boolean>(false);
    const [history, setHistory] = useState<ConversionResult[]>([]);

    const [settings, setSettings] = useState<AdvancedSettings>({
        crf: 23,
        resolution: "original",
        hardwareAccel: false,
        selectedEncoder: "auto",
        stripMetadata: true,
        audioBitrate: "192k",
        collisionPolicy: "autorename",
        maxParallelJobs: 4,
    });

    const loadSystemData = async () => {
        try {
            const [hw, sc, ps] = await Promise.all([
                api.detectHardware(),
                api.checkSidecars(),
                api.getPresets(),
            ]);
            setHardware(hw);
            setSidecars(sc);
            setPresets(ps);
        } catch (err) {
            console.error("Initialization error:", err);
        }
    };

    // Load persisted settings and history from local storage
    useEffect(() => {
        try {
            const savedSettings = localStorage.getItem("file2file_settings");
            if (savedSettings) {
                const parsed = JSON.parse(savedSettings);
                if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
                    setSettings(prev => ({ ...prev, ...parsed }));
                }
            }
            const savedHistory = localStorage.getItem("file2file_history");
            if (savedHistory) {
                const parsed = JSON.parse(savedHistory);
                if (Array.isArray(parsed)) {
                    setHistory(parsed.filter(item =>
                        item && typeof item.job_id === "string" && typeof item.input_path === "string" &&
                        typeof item.output_path === "string" && typeof item.success === "boolean" &&
                        typeof item.original_size_bytes === "number" && typeof item.converted_size_bytes === "number" &&
                        typeof item.elapsed_ms === "number"
                    ).slice(0, 50));
                }
            }
        } catch (e) {
            console.warn("Storage load warning:", e);
        }
        loadSystemData();
    }, []);

    // Persist settings
    useEffect(() => {
        try {
            localStorage.setItem("file2file_settings", JSON.stringify(settings));
        } catch {}
    }, [settings]);

    // Persist history
    useEffect(() => {
        try {
            localStorage.setItem("file2file_history", JSON.stringify(history.slice(0, 50).map(result => ({ ...result, download_url: undefined }))));
        } catch {}
    }, [history]);

    const handleAddFiles = async (newFiles: FileItem[]) => {
        if (newFiles.length === 0) return;
        setFiles(prev => [...prev, ...newFiles.filter(n => !prev.some(f => f.path === n.path))].slice(0, 256));

        try {
            const primaryExt = newFiles[0].extension;
            const formats = await api.getCompatibleTargets(primaryExt);
            setAvailableFormats(formats);
        } catch (err) {
            console.error("Format fetch error:", err);
        }
    };

    const handleSelectFormat = (format: FormatOption) => {
        setSelectedFormatOption(format);
        setCurrentStep("convert-box");
    };

    const handleSelectPreset = async (p: QuickPreset) => {
        // Find representative source extension
        let sourceExt = "";
        if (files.length > 0) {
            const match = files.find(f => f.category === p.from_category) || files[0];
            sourceExt = match.extension;
        } else {
            sourceExt = p.from_category === "document" ? "pdf" :
                        p.from_category === "image" ? "png" :
                        p.from_category === "video" ? "mp4" : "mp4";
        }

        try {
            const formats = await api.getCompatibleTargets(sourceExt);
            setAvailableFormats(formats);
            const target = formats.find(f => f.extension === p.to_format) || formats[0];
            if (target) {
                handleSelectFormat(target);
            }
        } catch (err) {
            console.error("Preset format fetch error:", err);
        }
    };

    const handleUpdateFileStatus = (id: string, status: FileItem["status"], result?: ConversionResult) => {
        setFiles(prev => prev.map(f => (f.id === id ? { ...f, status, result } : f)));
        if (result) {
            setHistory(prev => [result, ...prev.filter(h => h.job_id !== result.job_id)].slice(0, 50));
        }
    };

    return (
        <div className="h-screen flex flex-col bg-background text-text-primary overflow-hidden">
            <Header
                hardware={hardware}
                sidecars={sidecars}
                isSettingsOpen={isSettingsOpen}
                onToggleSettings={() => setIsSettingsOpen(!isSettingsOpen)}
                onRefreshHardware={loadSystemData}
                onOpenDiagnostics={() => setIsDiagnosticOpen(true)}
            />

            <main className="flex-1 flex overflow-hidden border-t border-border">
                {/* Main Workspace (Left) */}
                <div className="flex-1 overflow-y-auto custom-scrollbar p-6 lg:p-10 border-r border-border bg-background">
                    {currentStep === "upload" && (
                        <div className="max-w-4xl space-y-10">
                            <div className="space-y-1.5">
                                <h1 className="text-[28px] font-semibold text-text-primary tracking-tight">Convert files</h1>
                                <p className="text-[14px] text-text-secondary">
                                    Local, private conversion for video, audio, images, documents, data, and archives.
                                </p>
                            </div>

                            <QuickConverters
                                presets={presets}
                                onSelectPreset={handleSelectPreset}
                                sidecars={sidecars}
                            />

                            <DropZone
                                files={files}
                                onAddFiles={handleAddFiles}
                                onRemoveFile={(id) => setFiles(prev => prev.filter(f => f.id !== id))}
                                onClearFiles={() => setFiles([])}
                                onDirectConvert={handleSelectFormat}
                            />

                            {files.length > 0 && (
                                <div className="flex justify-start">
                                    <button
                                        onClick={() => setCurrentStep("select-format")}
                                        className="btn-primary flex items-center gap-2 group h-[48px] px-8"
                                    >
                                        <span>Choose output format</span>
                                        <ArrowRight className="w-4 h-4 group-hover:translate-x-1 transition-transform" />
                                    </button>
                                </div>
                            )}
                        </div>
                    )}

                    {currentStep === "select-format" && (
                        <div className="max-w-5xl">
                            <FormatSelectionPage
                                files={files}
                                availableFormats={availableFormats}
                                onSelectFormat={handleSelectFormat}
                                onBack={() => setCurrentStep("upload")}
                                sidecars={sidecars}
                            />
                        </div>
                    )}

                    {currentStep === "convert-box" && selectedFormatOption && (
                        <div className="h-full">
                            <ConversionStudio
                                files={files}
                                format={selectedFormatOption}
                                settings={settings}
                                hardware={hardware}
                                onBackToFormats={() => setCurrentStep("select-format")}
                                onResetToUpload={() => {
                                    setFiles([]);
                                    setCurrentStep("upload");
                                }}
                                onUpdateFileStatus={handleUpdateFileStatus}
                            />
                        </div>
                    )}
                </div>

                {/* Sidebar (Right) */}
                <aside className="hidden md:flex w-80 lg:w-96 flex-col shrink-0 bg-surface border-l border-border">
                    <div className="p-6 space-y-8 flex-1 overflow-y-auto custom-scrollbar">
                        {/* System Summary */}
                        <div className="space-y-4">
                            <h3 className="text-tiny uppercase tracking-widest text-text-muted font-bold">System Status</h3>
                            <div className="space-y-3">
                                <div className="flex items-start gap-3">
                                    <Shield className="w-3.5 h-3.5 text-success mt-0.5 shrink-0" />
                                    <div className="space-y-0.5">
                                        <p className="text-[12px] font-medium text-text-secondary">Local Execution</p>
                                        <p className="text-tiny text-text-muted leading-tight">
                                            {isTauri()
                                                ? "Conversions run on your device using installed local engines."
                                                : "Browser processing is performed locally; the PDF worker is included in this build."}
                                        </p>
                                    </div>
                                </div>
                                <div className="flex items-start gap-3">
                                    <Database className="w-3.5 h-3.5 text-accent mt-0.5 shrink-0" />
                                    <div className="space-y-0.5">
                                        <p className="text-[12px] font-medium text-text-secondary">Acceleration Engine</p>
                                        <p className="text-tiny text-text-muted leading-tight">
                                            {hardware?.hardware_acceleration_supported
                                                ? `Accelerated using ${hardware.recommended_encoder.toUpperCase()}`
                                                : "Standard software processing (multi-threaded CPU)"}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div className="h-[1px] bg-border" />

                        <HistoryPanel
                            history={history}
                            onClearHistory={() => setHistory([])}
                        />
                    </div>

                    {/* Compact Footer Status */}
                    <div className="h-[40px] border-t border-border px-4 flex items-center justify-between bg-surface-raised shrink-0">
                        <div className="flex gap-4">
                            <span className="text-tiny font-mono text-text-muted">CPU: {hardware?.cpu_cores || 0} Cores</span>
                            <span className="text-tiny font-mono text-text-muted">
                                SIDE: {sidecars?.all_ready ? "Ready" : "Partial"}
                            </span>
                        </div>
                        <div className="flex items-center gap-1.5">
                            <div className={`w-1.5 h-1.5 rounded-full ${sidecars?.all_ready ? "bg-success" : "bg-warning"}`} />
                            <span className="text-tiny font-medium text-text-muted uppercase tracking-tighter">
                                {sidecars?.all_ready ? "Operational" : "Limited Mode"}
                            </span>
                        </div>
                    </div>
                </aside>
            </main>

            <AdvancedSettingsDrawer
                isOpen={isSettingsOpen}
                onClose={() => setIsSettingsOpen(false)}
                settings={settings}
                onChangeSettings={setSettings}
                hardware={hardware}
            />

            <DiagnosticModal
                isOpen={isDiagnosticOpen}
                onClose={() => setIsDiagnosticOpen(false)}
                sidecars={sidecars}
            />
        </div>
    );
}

export default App;
