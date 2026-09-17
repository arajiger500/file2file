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
        hardwareAccel: true,
        selectedEncoder: "auto",
        stripMetadata: true,
        audioBitrate: "192k",
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

    useEffect(() => {
        loadSystemData();
    }, []);

    const handleAddFiles = async (newFiles: FileItem[]) => {
        if (newFiles.length === 0) return;
        setFiles([...files, ...newFiles]);
        try {
            const formats = await api.getCompatibleTargets(newFiles[0].extension);
            setAvailableFormats(formats);
        } catch (err) {
            console.error("Format fetch error:", err);
        }
    };

    const handleSelectFormat = (format: FormatOption) => {
        setSelectedFormatOption(format);
        setCurrentStep("convert-box");
    };

    const handleUpdateFileStatus = (id: string, status: FileItem["status"], result?: ConversionResult) => {
        setFiles(prev => prev.map(f => f.id === id ? { ...f, status, result } : f));
        if (result) setHistory(prev => [result, ...prev]);
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
                                <p className="text-[14px] text-text-secondary">Convert media, documents, images, data, and archives locally.</p>
                            </div>

                            <QuickConverters
                                presets={presets}
                                onSelectPreset={async (p) => {
                                    const formats = await api.getCompatibleTargets(p.to_format);
                                    setAvailableFormats(formats);
                                    handleSelectFormat(formats.find((f: FormatOption) => f.extension === p.to_format) || formats[0]);
                                }}
                            />

                            <DropZone
                                files={files}
                                onAddFiles={handleAddFiles}
                                onRemoveFile={(id) => setFiles(files.filter(f => f.id !== id))}
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
                                onResetToUpload={() => { setFiles([]); setCurrentStep("upload"); }}
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
                                    <Shield className="w-3.5 h-3.5 text-success mt-0.5" />
                                    <div className="space-y-0.5">
                                        <p className="text-[12px] font-medium text-text-secondary">Local Processing</p>
                                        <p className="text-tiny text-text-muted leading-tight">All conversions are performed locally. No files leave your device.</p>
                                    </div>
                                </div>
                                <div className="flex items-start gap-3">
                                    <Database className="w-3.5 h-3.5 text-accent mt-0.5" />
                                    <div className="space-y-0.5">
                                        <p className="text-[12px] font-medium text-text-secondary">Hardware Accelerated</p>
                                        <p className="text-tiny text-text-muted leading-tight">Using {hardware?.recommended_encoder.toUpperCase()} for optimal performance.</p>
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
                            <span className="text-tiny font-mono text-text-muted">MEM: OK</span>
                        </div>
                        <div className="flex items-center gap-1.5">
                            <div className="w-1.5 h-1.5 rounded-full bg-success" />
                            <span className="text-tiny font-medium text-text-muted uppercase tracking-tighter">Connected</span>
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
