import React, { useState, useEffect, useRef, useMemo } from "react";
import { ArrowLeft, Loader2, Download, AlertCircle, ExternalLink, RotateCcw, StopCircle, CheckCircle2 } from "lucide-react";
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
    onUpdateFileStatus,
}) => {
    const [isConverting, setIsConverting] = useState(false);
    const [results, setResults] = useState<ConversionResult[]>([]);
    const [startTime, setStartTime] = useState<number | null>(null);
    const [elapsedTime, setElapsedTime] = useState<number>(0);
    const [activeWorkerCount, setActiveWorkerCount] = useState<number>(0);

    const activeJobIdsRef = useRef<Set<string>>(new Set());
    const isCancelledRef = useRef<boolean>(false);

    // Live elapsed timer
    useEffect(() => {
        if (!isConverting || !startTime) return;
        const interval = setInterval(() => {
            setElapsedTime(Date.now() - startTime);
        }, 200);
        return () => clearInterval(interval);
    }, [isConverting, startTime]);

    const completedCount = files.filter(f => f.status === "completed" || f.status === "error" || f.status === "cancelled").length;
    const successfulCount = results.filter(r => r.success).length;
    const failedCount = results.filter(r => !r.success).length;
    const progress = files.length > 0 ? (completedCount / files.length) * 100 : 0;

    const runBatch = async (filesToProcess: FileItem[]) => {
        if (filesToProcess.length === 0) return;

        setIsConverting(true);
        isCancelledRef.current = false;
        activeJobIdsRef.current.clear();
        setStartTime(Date.now());

        const validatedQueue: FileItem[] = [];

        for (const file of filesToProcess) {
            if (isCancelledRef.current) break;
            try {
                const validation = await api.validateJob(file.path, format.extension);
                if (!validation.is_valid) {
                    const fail: ConversionResult = {
                        job_id: crypto.randomUUID(),
                        input_path: file.path,
                        output_path: "",
                        success: false,
                        original_size_bytes: file.size,
                        converted_size_bytes: 0,
                        elapsed_ms: 0,
                        error: validation.error || "Validation pre-flight failed",
                    };
                    onUpdateFileStatus(file.id, "error", fail);
                    setResults(prev => [...prev.filter(r => r.input_path !== file.path), fail]);
                } else {
                    validatedQueue.push(file);
                }
            } catch {
                validatedQueue.push(file);
            }
        }

        if (validatedQueue.length === 0 || isCancelledRef.current) {
            setIsConverting(false);
            return;
        }

        const concurrencyLimit = Math.min(settings.maxParallelJobs || 4, 4);
        let nextIndex = 0;

        const processFile = async (file: FileItem): Promise<ConversionResult> => {
            if (isCancelledRef.current) {
                onUpdateFileStatus(file.id, "cancelled");
                return {
                    job_id: crypto.randomUUID(),
                    input_path: file.path,
                    output_path: "",
                    success: false,
                    original_size_bytes: file.size,
                    converted_size_bytes: 0,
                    elapsed_ms: 0,
                    error: "Conversion cancelled",
                };
            }

            onUpdateFileStatus(file.id, "converting");
            const jobId = crypto.randomUUID();
            activeJobIdsRef.current.add(jobId);
            setActiveWorkerCount(prev => prev + 1);

            try {
                const res = await api.startConversion({
                    job_id: jobId,
                    input_path: file.path,
                    target_format: format.extension,
                    crf: settings.crf,
                    resolution: settings.resolution,
                    hardware_accel: settings.hardwareAccel,
                    selected_encoder: settings.selectedEncoder,
                    strip_metadata: settings.stripMetadata,
                    audio_bitrate: settings.audioBitrate,
                    collision_policy: settings.collisionPolicy,
                    rawFile: file.rawFile,
                });
                onUpdateFileStatus(file.id, res.success ? "completed" : "error", res);
                return res;
            } catch (err) {
                const fail: ConversionResult = {
                    job_id: jobId,
                    input_path: file.path,
                    output_path: "",
                    success: false,
                    original_size_bytes: file.size,
                    converted_size_bytes: 0,
                    elapsed_ms: 0,
                    error: String(err),
                };
                onUpdateFileStatus(file.id, "error", fail);
                return fail;
            } finally {
                activeJobIdsRef.current.delete(jobId);
                setActiveWorkerCount(prev => Math.max(0, prev - 1));
            }
        };

        const workerCount = Math.min(validatedQueue.length, concurrencyLimit);
        const workers = Array.from({ length: workerCount }, async () => {
            while (nextIndex < validatedQueue.length && !isCancelledRef.current) {
                const index = nextIndex++;
                const result = await processFile(validatedQueue[index]);
                setResults(prev => [...prev.filter(r => r.input_path !== result.input_path), result]);
            }
        });

        await Promise.all(workers);
        setIsConverting(false);
        setActiveWorkerCount(0);
    };

    const handleRun = () => {
        setResults([]);
        runBatch(files);
    };

    const handleCancel = async () => {
        isCancelledRef.current = true;
        const activeIds = Array.from(activeJobIdsRef.current);
        await Promise.all(activeIds.map(id => api.cancelJob(id).catch(() => {})));
        files.forEach(f => {
            if (f.status === "pending" || f.status === "converting") {
                onUpdateFileStatus(f.id, "cancelled");
            }
        });
        setIsConverting(false);
        setActiveWorkerCount(0);
    };

    const handleRetryFailed = () => {
        const failedFiles = files.filter(f => f.status === "error" || f.status === "cancelled");
        if (failedFiles.length > 0) {
            runBatch(failedFiles);
        }
    };

    const handleExportAll = async () => {
        if (!isTauri()) return;
        try {
            const { open } = await import("@tauri-apps/plugin-dialog");
            const selectedDir = await open({
                directory: true,
                multiple: false,
                title: "Select Output Directory",
            });

            if (selectedDir && typeof selectedDir === "string") {
                const successfulResults = results.filter(r => r.success && r.output_path);
                for (const res of successfulResults) {
                    const filename = res.output_path.split(/[\\/]/).pop();
                    if (filename) {
                        const dest = `${selectedDir}/${filename}`;
                        await api.copyFile(res.output_path, dest);
                    }
                }
                const { open: openShell } = await import("@tauri-apps/plugin-shell");
                await openShell(selectedDir);
            }
        } catch (err) {
            console.error("Export all error:", err);
        }
    };

    const handleShowInFolder = async (filePath: string) => {
        if (!isTauri() || !filePath) return;
        try {
            const parentDir = filePath.replace(/[\\/][^\\/]+$/, "");
            const { open } = await import("@tauri-apps/plugin-shell");
            await open(parentDir);
        } catch (err) {
            console.error("Open folder error:", err);
        }
    };

    const done = results.length === files.length && !isConverting && files.length > 0;

    const totalSavedBytes = useMemo(() => {
        return results.reduce((acc, curr) => {
            if (curr.success && curr.original_size_bytes > curr.converted_size_bytes) {
                return acc + (curr.original_size_bytes - curr.converted_size_bytes);
            }
            return acc;
        }, 0);
    }, [results]);

    const formatBytes = (bytes: number): string => {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
    };

    return (
        <div className="h-full flex flex-col gap-6">
            {/* Header / Nav */}
            <div className="flex items-center justify-between">
                <button
                    onClick={onBackToFormats}
                    disabled={isConverting}
                    className="btn-ghost h-[32px] px-2 flex items-center gap-2 -ml-2 disabled:opacity-50"
                    aria-label="Go back to format selection"
                >
                    <ArrowLeft className="w-4 h-4" />
                    <span>Back to formats</span>
                </button>

                <div className="flex items-center gap-3">
                    {isConverting && (
                        <>
                            <div className="flex items-center gap-2 text-tiny font-mono text-accent uppercase font-semibold">
                                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                <span>Converting {completedCount} / {files.length} ({(elapsedTime / 1000).toFixed(1)}s)</span>
                            </div>
                            <button
                                onClick={handleCancel}
                                className="btn-secondary h-[32px] px-3 text-[12px] text-error border-error/30 hover:bg-error/10 flex items-center gap-1.5"
                            >
                                <StopCircle className="w-3.5 h-3.5" />
                                <span>Cancel</span>
                            </button>
                        </>
                    )}
                </div>
            </div>

            <div className="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-8 min-h-0">
                {/* Main Content (Left) */}
                <div className="lg:col-span-8 flex flex-col gap-6 min-h-0">
                    {!done ? (
                        <div className="space-y-6 flex-1 flex flex-col min-h-0">
                            {/* Summary Config Header */}
                            <div className="space-y-1">
                                <h2 className="text-[22px] font-semibold text-text-primary tracking-tight">
                                    Convert to {format.extension.toUpperCase()}
                                </h2>
                                <p className="text-[13px] text-text-secondary">{files.length} files queued</p>
                            </div>

                            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 p-5 bg-surface border border-border rounded-lg">
                                <div className="space-y-1">
                                    <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Target Format</div>
                                    <div className="text-[13px] font-medium text-text-primary">{format.name}</div>
                                </div>
                                <div className="space-y-1">
                                    <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Engine</div>
                                    <div className="text-[13px] font-mono text-text-primary">{format.sidecar_engine}</div>
                                </div>
                                <div className="space-y-1">
                                    <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Acceleration</div>
                                    <div className="text-[13px] font-medium text-text-primary">
                                        {settings.hardwareAccel
                                            ? (hardware?.gpu_vendor ? `GPU (${hardware.gpu_vendor})` : "Hardware GPU")
                                            : "CPU Software"}
                                    </div>
                                </div>
                                <div className="space-y-1">
                                    <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Concurrency Limit</div>
                                    <div className="text-[13px] font-medium text-text-primary">{Math.min(settings.maxParallelJobs || 4, 4)} Workers</div>
                                </div>
                            </div>

                            {/* Execution Area */}
                            <div className="flex-1 flex flex-col items-center justify-center border border-border bg-surface-raised rounded-lg p-8">
                                {!isConverting ? (
                                    <div className="text-center space-y-6 max-w-sm">
                                        <div className="space-y-1.5">
                                            <h3 className="text-[17px] font-semibold text-text-primary">Ready to convert</h3>
                                            <p className="text-[13px] text-text-muted">Process {files.length} files locally and securely.</p>
                                        </div>
                                        <button
                                            onClick={handleRun}
                                            className="btn-primary w-full h-[48px] text-[15px] font-semibold shadow-sm"
                                        >
                                            Start conversion
                                        </button>
                                    </div>
                                ) : (
                                    <div className="w-full max-w-md space-y-5">
                                        <div className="text-center space-y-1.5">
                                            <h3 className="text-[16px] font-semibold text-text-primary">Converting files...</h3>
                                            <p className="text-[13px] text-text-muted font-mono">{completedCount} of {files.length} complete · {(elapsedTime / 1000).toFixed(1)}s</p>
                                        </div>
                                        <div className="space-y-2">
                                            <div className="h-2 w-full bg-surface border border-border rounded-full overflow-hidden">
                                                <div
                                                    className="h-full bg-accent transition-all duration-300"
                                                    style={{ width: `${progress}%` }}
                                                />
                                            </div>
                                            <div className="flex justify-between text-tiny font-mono text-text-muted uppercase">
                                                <span>Queue Progress</span>
                                                <span>{Math.round(progress)}%</span>
                                            </div>
                                        </div>
                                    </div>
                                )}
                            </div>
                        </div>
                    ) : (
                        <div className="space-y-6 flex-1 flex flex-col min-h-0">
                            {/* Completion Header */}
                            <div className="flex items-center justify-between">
                                <div className="space-y-1">
                                    <h2 className="text-[22px] font-semibold text-text-primary tracking-tight">Conversion complete</h2>
                                    <p className="text-[13px] text-text-secondary">
                                        {successfulCount} successful · {failedCount} failed · {(elapsedTime / 1000).toFixed(1)}s total
                                    </p>
                                </div>
                                <div className="flex gap-2">
                                    {failedCount > 0 && (
                                        <button
                                            onClick={handleRetryFailed}
                                            className="btn-secondary flex items-center gap-1.5 text-warning"
                                        >
                                            <RotateCcw className="w-3.5 h-3.5" />
                                            <span>Retry Failed</span>
                                        </button>
                                    )}
                                    <button onClick={onResetToUpload} className="btn-secondary">New conversion</button>
                                    {isTauri() && (
                                        <button onClick={handleExportAll} className="btn-primary">Export All</button>
                                    )}
                                </div>
                            </div>

                            {/* Summary Metrics */}
                            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 p-5 bg-surface border border-border rounded-lg">
                                <div className="space-y-1">
                                    <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Elapsed Time</div>
                                    <div className="text-[13px] font-medium text-text-primary">{(elapsedTime / 1000).toFixed(1)} s</div>
                                </div>
                                <div className="space-y-1">
                                    <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Input Size</div>
                                    <div className="text-[13px] font-medium text-text-primary">
                                        {formatBytes(files.reduce((a, b) => a + b.size, 0))}
                                    </div>
                                </div>
                                <div className="space-y-1">
                                    <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Output Size</div>
                                    <div className="text-[13px] font-medium text-text-primary">
                                        {formatBytes(results.reduce((a, b) => a + (b.converted_size_bytes || 0), 0))}
                                    </div>
                                </div>
                                <div className="space-y-1">
                                    <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Saved Storage</div>
                                    <div className="text-[13px] font-semibold text-success">
                                        {formatBytes(totalSavedBytes)}
                                    </div>
                                </div>
                            </div>

                            {/* Results List */}
                            <div className="flex-1 overflow-y-auto custom-scrollbar pr-2 space-y-2">
                                {results.map((res) => (
                                    <div key={res.job_id} className="h-[60px] flex items-center justify-between px-4 bg-surface border border-border rounded-lg group">
                                        <div className="flex items-center gap-3 truncate">
                                            {res.success ? (
                                                <CheckCircle2 className="w-4 h-4 text-success shrink-0" />
                                            ) : (
                                                <AlertCircle className="w-4 h-4 text-error shrink-0" />
                                            )}
                                            <div className="truncate">
                                                <div className="text-[13px] font-medium text-text-primary truncate">
                                                    {res.output_path ? res.output_path.split(/[\\/]/).pop() : res.input_path.split(/[\\/]/).pop()}
                                                </div>
                                                {res.success ? (
                                                    <div className="text-tiny font-mono text-text-muted uppercase">
                                                        {formatBytes(res.original_size_bytes)} → {formatBytes(res.converted_size_bytes)} · {res.elapsed_ms}ms
                                                    </div>
                                                ) : (
                                                    <div className="text-tiny font-mono text-error uppercase truncate max-w-md">{res.error}</div>
                                                )}
                                            </div>
                                        </div>
                                        <div className="flex gap-2">
                                            {res.success ? (
                                                <>
                                                    <button onClick={() => downloadFile(res)} className="p-2 text-text-muted hover:text-text-primary hover:bg-surface-raised rounded transition-colors" title="Download">
                                                        <Download className="w-4 h-4" />
                                                    </button>
                                                    {isTauri() && res.output_path && (
                                                        <button onClick={() => handleShowInFolder(res.output_path)} className="p-2 text-text-muted hover:text-text-primary hover:bg-surface-raised rounded transition-colors" title="Show in folder">
                                                            <ExternalLink className="w-4 h-4" />
                                                        </button>
                                                    )}
                                                </>
                                            ) : (
                                                <span className="text-tiny text-error font-mono px-2 py-1 bg-error/10 rounded">Failed</span>
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
                        <span className="text-tiny font-bold uppercase tracking-widest text-text-muted">Queue ({files.length})</span>
                        {isConverting && <Loader2 className="w-3.5 h-3.5 text-accent animate-spin" />}
                    </div>

                    <div className="flex-1 overflow-y-auto custom-scrollbar divide-y divide-border">
                        {files.map(f => (
                            <div key={f.id} className="px-4 py-3 flex items-center justify-between transition-colors">
                                <div className="flex items-center gap-2.5 truncate">
                                    <div className={`w-1.5 h-1.5 rounded-full ${
                                        f.status === "completed" ? "bg-success" :
                                        f.status === "converting" ? "bg-accent animate-pulse" :
                                        f.status === "error" ? "bg-error" :
                                        f.status === "cancelled" ? "bg-warning" :
                                        "bg-text-disabled"
                                    }`} />
                                    <span className={`text-[12px] truncate ${f.status === "pending" ? "text-text-muted" : "text-text-primary"}`}>
                                        {f.name}
                                    </span>
                                </div>
                                <span className={`text-tiny font-mono uppercase ${
                                    f.status === "completed" ? "text-success" :
                                    f.status === "converting" ? "text-accent font-semibold" :
                                    f.status === "error" ? "text-error" :
                                    "text-text-disabled"
                                }`}>
                                    {f.status}
                                </span>
                            </div>
                        ))}
                    </div>

                    {isConverting && (
                        <div className="p-3 bg-surface-raised border-t border-border space-y-2">
                            <div className="flex justify-between text-tiny font-mono text-text-muted uppercase">
                                <span>Active worker threads</span>
                                <span>{activeWorkerCount} / {Math.min(settings.maxParallelJobs || 4, 4)} active</span>
                            </div>
                            <div className="flex gap-1.5">
                                {Array.from({ length: Math.min(settings.maxParallelJobs || 4, 4) }).map((_, i) => (
                                    <div key={i} className="h-1 flex-1 bg-surface border border-border rounded-full overflow-hidden">
                                        <div
                                            className={`h-full ${i < activeWorkerCount ? "bg-accent" : "bg-transparent"}`}
                                        />
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
