import { useEffect, useState } from "react";
import { Header } from "./components/Header";
import { DropZone } from "./components/DropZone";
import { FormatSelectionPage } from "./components/FormatSelectionPage";
import { ConversionStudio } from "./components/ConversionStudio";
import { AdvancedSettingsDrawer } from "./components/AdvancedSettingsDrawer";
import { QuickConverters } from "./components/QuickConverters";
import { HistoryPanel } from "./components/HistoryPanel";
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
import { Shield, Zap, Globe } from "lucide-react";

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
            setCurrentStep("select-format");
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
        <div className="min-h-screen flex flex-col bg-zinc-950 selection:bg-blue-500/30">
            <Header
                hardware={hardware}
                sidecars={sidecars}
                isSettingsOpen={isSettingsOpen}
                onToggleSettings={() => setIsSettingsOpen(!isSettingsOpen)}
                onRefreshHardware={loadSystemData}
            />

            <main className="flex-1 w-full max-w-[1400px] mx-auto px-6 py-8">
                {currentStep === "upload" && (
                    <div className="grid grid-cols-1 xl:grid-cols-12 gap-8 items-start">
                        <div className="xl:col-span-8 space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-700">
                            <div className="space-y-2">
                                <h2 className="text-4xl font-bold tracking-tight text-white">Universal Transcoder</h2>
                                <p className="text-zinc-400 text-lg">High-fidelity conversions for any workflow.</p>
                            </div>

                            <QuickConverters
                                presets={presets}
                                onSelectPreset={async (p) => {
                                    const formats = await api.getCompatibleTargets(p.to_format);
                                    setAvailableFormats(formats);
                                    handleSelectFormat(formats.find(f => f.extension === p.to_format) || formats[0]);
                                }}
                            />

                            <DropZone
                                files={files}
                                onAddFiles={handleAddFiles}
                                onRemoveFile={(id) => setFiles(files.filter(f => f.id !== id))}
                                onClearFiles={() => setFiles([])}
                            />
                        </div>

                        <div className="xl:col-span-4 space-y-6">
                            <div className="glass-card p-6 space-y-5 bg-gradient-to-br from-zinc-900/80 to-zinc-950/80">
                                <div className="flex items-center gap-3 text-zinc-100">
                                    <div className="p-2 rounded-lg bg-blue-500/10 border border-blue-500/20">
                                        <Shield className="w-5 h-5 text-blue-400" />
                                    </div>
                                    <div>
                                        <h3 className="font-bold">Air-Gapped Privacy</h3>
                                        <p className="text-xs text-zinc-500">100% On-Device Processing</p>
                                    </div>
                                </div>
                                <div className="grid grid-cols-2 gap-3 text-[11px]">
                                    <div className="p-3 rounded-xl bg-zinc-900/50 border border-zinc-800 flex items-center gap-2">
                                        <Zap className="w-4 h-4 text-amber-400" />
                                        <span>GPU Accelerated</span>
                                    </div>
                                    <div className="p-3 rounded-xl bg-zinc-900/50 border border-zinc-800 flex items-center gap-2">
                                        <Globe className="w-4 h-4 text-emerald-400" />
                                        <span>Offline Core</span>
                                    </div>
                                </div>
                            </div>

                            <HistoryPanel
                                history={history}
                                onClearHistory={() => setHistory([])}
                            />
                        </div>
                    </div>
                )}

                {currentStep === "select-format" && (
                    <div className="animate-in fade-in zoom-in-95 duration-500">
                        <FormatSelectionPage
                            files={files}
                            availableFormats={availableFormats}
                            onSelectFormat={handleSelectFormat}
                            onBack={() => setCurrentStep("upload")}
                        />
                    </div>
                )}

                {currentStep === "convert-box" && selectedFormatOption && (
                    <div className="animate-in fade-in slide-in-from-right-8 duration-500">
                        <ConversionStudio
                            files={files}
                            format={selectedFormatOption}
                            settings={settings}
                            onChangeSettings={setSettings}
                            hardware={hardware}
                            onBackToFormats={() => setCurrentStep("select-format")}
                            onResetToUpload={() => { setFiles([]); setCurrentStep("upload"); }}
                            onUpdateFileStatus={handleUpdateFileStatus}
                        />
                    </div>
                )}
            </main>

            <AdvancedSettingsDrawer
                isOpen={isSettingsOpen}
                onClose={() => setIsSettingsOpen(false)}
                settings={settings}
                onChangeSettings={setSettings}
                hardware={hardware}
            />
        </div>
    );
}

export default App;
