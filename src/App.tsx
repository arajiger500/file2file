import { isTauri } from "./services/api";
import { useEffect, useRef, useState } from "react";
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
    DEFAULT_SETTINGS,
    getEngineIssue,
    intersectFormatGroups,
    sanitizeSettings,
} from "./services/conversion-planning";
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
    const [systemError, setSystemError] = useState<string | null>(null);
    const [formatError, setFormatError] = useState<string | null>(null);
    const [formatsLoading, setFormatsLoading] = useState(false);
    const formatRequestRef = useRef(0);

    const [settings, setSettings] = useState<AdvancedSettings>(DEFAULT_SETTINGS);

    const loadSystemData = async () => {
        setSystemError(null);
        const [hw, sc, ps] = await Promise.allSettled([
            api.detectHardware(),
            api.checkSidecars(),
            api.getPresets(),
        ]);
        if (hw.status === "fulfilled") setHardware(hw.value);
        if (sc.status === "fulfilled") setSidecars(sc.value);
        if (ps.status === "fulfilled") setPresets(ps.value);
        const failures = [hw, sc, ps].filter(result => result.status === "rejected");
        if (failures.length > 0) {
            console.error("Initialization errors:", failures);
            setSystemError(`${failures.length} system check${failures.length === 1 ? "" : "s"} could not be loaded. Refresh diagnostics to retry.`);
        }
    };

    // Load persisted settings and history from local storage
    useEffect(() => {
        try {
            const savedSettings = localStorage.getItem("file2file_settings");
            if (savedSettings) {
                const parsed = JSON.parse(savedSettings);
                if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
                    setSettings(sanitizeSettings(parsed));
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

    useEffect(() => {
        if (!hardware) return;
        setSettings(current => {
            const encoderAvailable = current.selectedEncoder === "auto" ||
                hardware.available_encoders.some(encoder => encoder.id === current.selectedEncoder);
            const hardwareAccel = current.hardwareAccel && hardware.hardware_acceleration_supported;
            if (encoderAvailable && hardwareAccel === current.hardwareAccel) return current;
            return {
                ...current,
                hardwareAccel,
                selectedEncoder: encoderAvailable ? current.selectedEncoder : "auto",
            };
        });
    }, [hardware]);

    // Persist history
    useEffect(() => {
        try {
            localStorage.setItem("file2file_history", JSON.stringify(history.slice(0, 50).map(result => ({ ...result, download_url: undefined }))));
        } catch {}
    }, [history]);

    // A mixed batch may only choose targets supported by every selected source.
    // Request sequencing prevents a slow lookup for an older selection from winning.
    useEffect(() => {
        const requestId = ++formatRequestRef.current;
        if (files.length === 0) {
            setAvailableFormats([]);
            setFormatError(null);
            setFormatsLoading(false);
            return;
        }
        setFormatsLoading(true);
        setFormatError(null);
        const extensions = Array.from(new Set(files.map(file => file.extension.toLowerCase())));
        void Promise.all(extensions.map(extension => api.getCompatibleTargets(extension)))
            .then(formatGroups => {
                if (requestId !== formatRequestRef.current) return;
                const commonFormats = intersectFormatGroups(formatGroups);
                setAvailableFormats(commonFormats);
                if (commonFormats.length === 0) {
                    setFormatError("These files do not share a safe output format. Split them into compatible batches.");
                }
            })
            .catch(error => {
                if (requestId === formatRequestRef.current) {
                    setAvailableFormats([]);
                    setFormatError(`Compatible formats could not be loaded: ${String(error)}`);
                }
            })
            .finally(() => {
                if (requestId === formatRequestRef.current) setFormatsLoading(false);
            });
    }, [files]);

    const handleAddFiles = (newFiles: FileItem[]) => {
        if (newFiles.length === 0) return;
        setFiles(prev => [...prev, ...newFiles.filter(n => !prev.some(f => f.path === n.path))].slice(0, 256));
    };

    const handleSelectFormat = (format: FormatOption) => {
        if (!availableFormats.some(candidate => candidate.extension === format.extension)) {
            setFormatError(`.${format.extension} is not compatible with every file in this batch.`);
            return;
        }
        if (getEngineIssue(format, sidecars)) {
            setSystemError(`The local engine required for .${format.extension} is missing or unusable. Open Engine diagnostics for details.`);
            setIsDiagnosticOpen(true);
            return;
        }
        setSelectedFormatOption(format);
        setCurrentStep("convert-box");
    };

    const handleSelectPreset = async (p: QuickPreset) => {
        if (files.length === 0) {
            setSystemError(`Add ${p.from_category === "image" ? "an" : "a"} ${p.from_category} file before using this shortcut.`);
            return;
        }
        if (!files.some(file => file.category === p.from_category)) {
            setSystemError(`This shortcut requires ${p.from_category} input.`);
            return;
        }
        // Find representative source extension
        let sourceExt = "";
        const match = files.find(f => f.category === p.from_category) || files[0];
        sourceExt = match.extension;

        try {
            const formats = await api.getCompatibleTargets(sourceExt);
            const target = formats.find(f => f.extension === p.to_format && availableFormats.some(common => common.extension === f.extension));
            if (target) {
                handleSelectFormat(target);
            } else {
                setSystemError(`.${p.to_format} is not compatible with every file in this batch.`);
            }
        } catch (err) {
            console.error("Preset format fetch error:", err);
        }
    };

    const handleUpdateFileStatus = (id: string, status: FileItem["status"], result?: ConversionResult) => {
        const recordedResult = result
            ? { ...result, completed_at: result.completed_at || new Date().toISOString() }
            : undefined;
        setFiles(prev => prev.map(f => (f.id === id ? { ...f, status, result: recordedResult } : f)));
        if (recordedResult) {
            setHistory(prev => [recordedResult, ...prev.filter(h => h.job_id !== recordedResult.job_id)].slice(0, 50));
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
                                compatibleFormats={availableFormats}
                            />

                            {(systemError || formatError) && (
                                <div role="alert" className="border border-warning/40 bg-warning/10 px-4 py-3 rounded-lg text-[13px] text-warning">
                                    {formatError || systemError}
                                </div>
                            )}

                            {files.length > 0 && (
                                <div className="flex justify-start">
                                    <button
                                        onClick={() => setCurrentStep("select-format")}
                                        disabled={formatsLoading || availableFormats.length === 0}
                                        className="btn-primary flex items-center gap-2 group h-[48px] px-8"
                                    >
                                        <span>{formatsLoading ? "Checking compatibility…" : "Choose output format"}</span>
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
                                isLoading={formatsLoading}
                                error={formatError}
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
