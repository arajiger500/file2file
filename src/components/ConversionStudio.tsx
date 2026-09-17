import React, { useState, useMemo } from "react";
import { ArrowLeft, Loader2, Download, AlertCircle, ExternalLink } from "lucide-react";
import { FileItem, FormatOption, AdvancedSettings, ConversionResult, HardwareInfo } from "../types";
import { api, downloadFile, isTauri } from "../services/api";

interface ConversionStudioProps {
    files: FileItem[];
    format: FormatOption;
    settings: AdvancedSettings;
    hardware: HardwareInfo | null;
    onBackToFormats: () => void;
    onResetToUpload: () => void;
    onUpdateFileStatus: (id: string, status: FileItem["status"], result?: ConversionResult) => void;
}

export const ConversionStudio: React.FC<ConversionStudioProps> = ({
    files,
    format,
    settings,
    hardware,
    onBackToFormats,
    onResetToUpload,
    onUpdateFileStatus
}) => {
    const [isConverting, setIsConverting] = useState(false);
    const [results, setResults] = useState<ConversionResult[]>([]);
    const [startTime, setStartTime] = useState<number | null>(null);
    const [currentTime, setCurrentTime] = useState<number | null>(null);

    const completedCount = files.filter(f => f.status === "completed" || f.status === "error").length;
    const progress = files.length > 0 ? (completedCount / files.length) * 100 : 0;

    const handleRun = async () => {
        setIsConverting(true);
        setResults([]);
        setStartTime(Date.now());

        const fileQueue = [...files];
        const validatedQueue: FileItem[] = [];

        for (const file of fileQueue) {
            try {
                const validation = await api.validateJob(file.path, format.extension);
                if (!validation.is_valid) {
                    const fail: ConversionResult = {
                        job_id: Math.random().toString(),
                        input_path: file.path,
                        output_path: file.path,
                        success: false,
                        original_size_bytes: file.size,
                        converted_size_bytes: 0,
                        elapsed_ms: 0,
                        error: validation.error || "Validation failed"
                    };
                    onUpdateFileStatus(file.id, "error", fail);
                    setResults(prev => [...prev, fail]);
                } else {
                    validatedQueue.push(file);
                }
            } catch (err) {
                validatedQueue.push(file);
            }
        }

        if (validatedQueue.length === 0 && fileQueue.length > 0) {
            setIsConverting(false);
            setCurrentTime(Date.now());
            return;
        }

        const concurrencyLimit = hardware?.cpu_cores || 4;
        let activeIndex = 0;

        const processFile = async (file: FileItem): Promise<ConversionResult> => {
            onUpdateFileStatus(file.id, "converting");
            try {
                const res = await api.startConversion({
                    input_path: file.path,
                    target_format: format.extension,
                    crf: settings.crf,
                    resolution: settings.resolution,
                    hardware_accel: settings.hardwareAccel,
                    selected_encoder: settings.selectedEncoder,
                    strip_metadata: settings.stripMetadata,
                    audio_bitrate: settings.audioBitrate,
                    rawFile: file.rawFile,
                });
                onUpdateFileStatus(file.id, res.success ? "completed" : "error", res);
                return res;
            } catch (err) {
                const fail: ConversionResult = {
                    job_id: Math.random().toString(),
                    input_path: file.path,
                    output_path: file.path,
                    success: false,
                    original_size_bytes: file.size,
                    converted_size_bytes: 0,
                    elapsed_ms: 0,
                    error: String(err)
                };
                onUpdateFileStatus(file.id, "error", fail);
                return fail;
            }
        };

        const workers = Array.from({ length: Math.min(validatedQueue.length, concurrencyLimit) }, async () => {
            while (activeIndex < validatedQueue.length) {
                const index = activeIndex++;
                const result = await processFile(validatedQueue[index]);
                setResults(prev => [...prev, result]);
            }
        });

        await Promise.all(workers);
        setIsConverting(false);
        setCurrentTime(Date.now());
    };

    const handleExportAll = async () => {
        if (!isTauri()) return;
        try {
            const { open } = await import("@tauri-apps/plugin-dialog");
            const selectedDir = await open({
                directory: true,
                multiple: false,
                title: "Select Output Directory"
            });

            if (selectedDir && typeof selectedDir === "string") {
                const successfulResults = results.filter(r => r.success);
                for (const res of successfulResults) {
                    const filename = res.output_path.split(/[\\/]/).pop();
                    if (filename) {
                        const dest = `${selectedDir}/${filename}`;
                        await (await import("@tauri-apps/api/core")).invoke("copy_file", { src: res.output_path, dest });
                    }
                }
                await (await import("@tauri-apps/plugin-shell")).open(selectedDir);
            }
        } catch (err) {
            console.error("Export all error:", err);
        }
    };

    const done = results.length === files.length && !isConverting && files.length > 0;
    const totalElapsed = startTime && currentTime ? (currentTime - startTime) : (startTime ? Date.now() - startTime : 0);

    const totalSavedBytes = useMemo(() => {
        return results.reduce((acc, curr) => {
            if (curr.success && curr.original_size_bytes > curr.converted_size_bytes) {
                return acc + (curr.original_size_bytes - curr.converted_size_bytes);
            }
            return acc;
        }, 0);
    }, [results]);

    return (
        <div className="h-full flex flex-col gap-8">
            {/* Header / Nav */}
            <div className="flex items-center justify-between">
                <button
                    onClick={onBackToFormats}
                    className="btn-ghost h-[32px] px-2 flex items-center gap-2 -ml-2"
                >
                    <ArrowLeft className="w-4 h-4" />
                    <span>Back to formats</span>
                </button>

                {isConverting && (
                    <div className="flex items-center gap-2 text-tiny font-mono text-accent uppercase font-semibold">
                        <Loader2 className="w-3.5 h-3.5 animate-spin" />
                        <span>Converting {completedCount} of {files.length}</span>
                    </div>
                )}
            </div>

            <div className="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-10 min-h-0">
                {/* Main Content (Left) */}
                <div className="lg:col-span-8 flex flex-col gap-8 min-h-0">
                    {!done ? (
                        <div className="space-y-8 flex-1 flex flex-col min-h-0">
                            {/* Config Summary */}
                            <div className="space-y-6">
                                <div className="space-y-1">
                                    <h2 className="text-[24px] font-semibold text-text-primary tracking-tight">Convert to {format.extension.toUpperCase()}</h2>
                                    <p className="text-[14px] text-text-secondary">{files.length} files selected</p>
                                </div>

                                <div className="grid grid-cols-2 md:grid-cols-4 gap-6 p-6 bg-surface border border-border rounded-lg">
                                    <div className="space-y-1">
                                        <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Format</div>
                                        <div className="text-[14px] font-medium text-text-primary">{format.name}</div>
                                    </div>
                                    <div className="space-y-1">
                                        <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Engine</div>
                                        <div className="text-[14px] font-medium text-text-primary font-mono">{format.sidecar_engine}</div>
                                    </div>
                                    <div className="space-y-1">
                                        <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Acceleration</div>
                                        <div className="text-[14px] font-medium text-text-primary">{settings.hardwareAccel ? "Hardware" : "Software"}</div>
                                    </div>
                                    <div className="space-y-1">
                                        <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Workers</div>
                                        <div className="text-[14px] font-medium text-text-primary">{hardware?.cpu_cores || 4} Parallel</div>
                                    </div>
                                </div>
                            </div>

                            {/* Execution Area */}
                            <div className="flex-1 flex flex-col items-center justify-center border border-border bg-surface-raised rounded-lg p-10">
                                {!isConverting ? (
                                    <div className="text-center space-y-8">
                                        <div className="space-y-2">
                                            <h3 className="text-[18px] font-semibold text-text-primary">Ready to convert</h3>
                                            <p className="text-[13px] text-text-muted">Starting a batch conversion of {files.length} files.</p>
                                        </div>
                                        <button
                                            onClick={handleRun}
                                            className="btn-primary h-[52px] px-10 text-[15px] font-semibold"
                                        >
                                            Start conversion
                                        </button>
                                    </div>
                                ) : (
                                    <div className="w-full max-w-md space-y-6">
                                        <div className="text-center space-y-2">
                                            <h3 className="text-[16px] font-semibold text-text-primary">Converting files...</h3>
                                            <p className="text-[13px] text-text-muted font-mono">{completedCount} / {files.length} complete</p>
                                        </div>
                                        <div className="space-y-2">
                                            <div className="h-2 w-full bg-surface border border-border rounded-full overflow-hidden">
                                                <div
                                                    className="h-full bg-accent transition-all duration-300"
                                                    style={{ width: `${progress}%` }}
                                                />
                                            </div>
                                            <div className="flex justify-between text-tiny font-mono text-text-muted uppercase">
                                                <span>Overall progress</span>
                                                <span>{Math.round(progress)}%</span>
                                            </div>
                                        </div>
                                    </div>
                                )}
                            </div>
                        </div>
                    ) : (
                        <div className="space-y-8 flex-1 flex flex-col min-h-0">
                            {/* Completion Header */}
                            <div className="flex items-center justify-between">
                                <div className="space-y-1">
                                    <h2 className="text-[24px] font-semibold text-text-primary tracking-tight">Conversion complete</h2>
                                    <p className="text-[14px] text-text-secondary">
                                        {results.filter(r => r.success).length} successful · {results.filter(r => !r.success).length} failed
                                    </p>
                                </div>
                                <div className="flex gap-2">
                                    <button onClick={onResetToUpload} className="btn-secondary">New conversion</button>
                                    {isTauri() && <button onClick={handleExportAll} className="btn-primary">Open output folder</button>}
                                </div>
                            </div>

                            {/* Summary Metrics */}
                            <div className="grid grid-cols-2 md:grid-cols-4 gap-6 p-6 bg-surface border border-border rounded-lg">
                                <div className="space-y-1">
                                    <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Total time</div>
                                    <div className="text-[14px] font-medium text-text-primary">{(totalElapsed / 1000).toFixed(1)} s</div>
                                </div>
                                <div className="space-y-1">
                                    <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Input size</div>
                                    <div className="text-[14px] font-medium text-text-primary">
                                        {(files.reduce((a, b) => a + b.size, 0) / (1024*1024*1024)).toFixed(2)} GB
                                    </div>
                                </div>
                                <div className="space-y-1">
                                    <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Output size</div>
                                    <div className="text-[14px] font-medium text-text-primary">
                                        {(results.reduce((a, b) => a + (b.converted_size_bytes || 0), 0) / (1024*1024*1024)).toFixed(2)} GB
                                    </div>
                                </div>
                                <div className="space-y-1">
                                    <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Storage saved</div>
                                    <div className="text-[14px] font-semibold text-success">
                                        {(totalSavedBytes / (1024*1024)).toFixed(1)} MB
                                    </div>
                                </div>
                            </div>

                            {/* Results List */}
                            <div className="flex-1 overflow-y-auto custom-scrollbar pr-2 space-y-2">
                                {results.map((res, i) => (
                                    <div key={i} className="h-[64px] flex items-center justify-between px-4 bg-surface border border-border rounded-lg group">
                                        <div className="flex items-center gap-4 truncate">
                                            <div className={`w-2 h-2 rounded-full ${res.success ? "bg-success" : "bg-error"}`} />
                                            <div className="truncate">
                                                <div className="text-[13px] font-medium text-text-primary truncate">{res.output_path.split(/[\\/]/).pop()}</div>
                                                {res.success ? (
                                                    <div className="text-tiny font-mono text-text-muted uppercase">
                                                        {(res.original_size_bytes / (1024*1024)).toFixed(1)}MB → {(res.converted_size_bytes / (1024*1024)).toFixed(1)}MB · {res.elapsed_ms}ms
                                                    </div>
                                                ) : (
                                                    <div className="text-tiny font-mono text-error uppercase truncate max-w-md">{res.error}</div>
                                                )}
                                            </div>
                                        </div>
                                        <div className="flex gap-2">
                                            {res.success ? (
                                                <>
                                                    <button onClick={() => downloadFile(res)} className="p-2 text-text-muted hover:text-text-primary hover:bg-surface-raised transition-colors" title="Download">
                                                        <Download className="w-4.5 h-4.5" />
                                                    </button>
                                                    {isTauri() && (
                                                        <button className="p-2 text-text-muted hover:text-text-primary hover:bg-surface-raised transition-colors" title="Show in folder">
                                                            <ExternalLink className="w-4.5 h-4.5" />
                                                        </button>
                                                    )}
                                                </>
                                            ) : (
                                                <button className="p-2 text-text-muted hover:text-text-primary transition-colors" title="View details">
                                                    <AlertCircle className="w-4.5 h-4.5" />
                                                </button>
                                            )}
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}
                </div>

                {/* Queue / Queue Status (Right) */}
                <div className="lg:col-span-4 flex flex-col min-h-0 bg-surface border border-border rounded-lg overflow-hidden">
                    <div className="px-4 py-3 border-b border-border bg-surface-raised flex items-center justify-between">
                        <span className="text-tiny font-bold uppercase tracking-widest text-text-muted">Queue Status</span>
                        {isConverting && <Loader2 className="w-3.5 h-3.5 text-accent animate-spin" />}
                    </div>

                    <div className="flex-1 overflow-y-auto custom-scrollbar divide-y divide-border">
                        {files.map(f => (
                            <div key={f.id} className="px-4 py-3 flex items-center justify-between transition-colors">
                                <div className="flex items-center gap-3 truncate">
                                    <div className={`w-1 h-3 rounded-full ${
                                        f.status === "completed" ? "bg-success" :
                                        f.status === "converting" ? "bg-accent" :
                                        f.status === "error" ? "bg-error" :
                                        "bg-text-disabled"
                                    }`} />
                                    <span className={`text-[12px] truncate ${f.status === "pending" ? "text-text-muted" : "text-text-secondary"}`}>
                                        {f.name}
                                    </span>
                                </div>
                                <span className="text-tiny font-mono text-text-muted uppercase">
                                    {f.status}
                                </span>
                            </div>
                        ))}
                    </div>

                    {isConverting && (
                        <div className="p-4 bg-surface-raised border-t border-border space-y-3">
                            <div className="flex justify-between text-tiny font-mono text-text-muted uppercase">
                                <span>Active workers</span>
                                <span>{hardware?.cpu_cores || 4} threads</span>
                            </div>
                            <div className="flex gap-1.5">
                                {Array.from({ length: hardware?.cpu_cores || 4 }).map((_, i) => (
                                    <div key={i} className="h-1 flex-1 bg-accent/30 rounded-full overflow-hidden">
                                        <div className="h-full bg-accent" style={{ opacity: 0.5 }} />
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};
