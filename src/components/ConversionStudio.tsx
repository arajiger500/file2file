import React, { useState, useEffect, useRef, useMemo } from "react";
import { ArrowLeft, Loader2, Download, AlertCircle, ExternalLink, RotateCcw, StopCircle, CheckCircle2, FolderOpen, X, ClipboardCopy, RefreshCw } from "lucide-react";
import { FileItem, FormatOption, AdvancedSettings, ConversionResult, HardwareInfo, BatchValidationItem } from "../types";
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
    const runningRef = useRef(false);
    const [operationError, setOperationError] = useState<string | null>(null);
    const [isCancelling, setIsCancelling] = useState(false);
    const [isConverting, setIsConverting] = useState(false);
    const [results, setResults] = useState<ConversionResult[]>([]);
    const [startTime, setStartTime] = useState<number | null>(null);
    const [elapsedTime, setElapsedTime] = useState<number>(0);
    const [activeWorkerCount, setActiveWorkerCount] = useState<number>(0);
    const [outputDirectory, setOutputDirectory] = useState<string | null>(null);
    const [preflightItems, setPreflightItems] = useState<BatchValidationItem[]>([]);
    const [preflightStatus, setPreflightStatus] = useState<"loading" | "ready" | "error">("loading");
    const [preflightError, setPreflightError] = useState<string | null>(null);
    const [preflightNonce, setPreflightNonce] = useState(0);
    const [reportCopied, setReportCopied] = useState(false);

    const activeJobIdsRef = useRef<Set<string>>(new Set());
    const validationJobIdRef = useRef<string | null>(null);
    const isCancelledRef = useRef<boolean>(false);
    const blobUrlsRef = useRef<Set<string>>(new Set());
    const reportCopiedTimerRef = useRef<number | null>(null);

    useEffect(() => () => {
        blobUrlsRef.current.forEach(url => URL.revokeObjectURL(url));
        blobUrlsRef.current.clear();
        if (reportCopiedTimerRef.current !== null) {
            window.clearTimeout(reportCopiedTimerRef.current);
        }
    }, []);

    const preflightKey = useMemo(
        () => JSON.stringify([format.extension, ...files.map(file => file.path)]),
        [files, format.extension],
    );

    useEffect(() => {
        let disposed = false;
        const jobId = crypto.randomUUID();
        validationJobIdRef.current = jobId;
        setPreflightStatus("loading");
        setPreflightError(null);
        setPreflightItems([]);

        void api.validateJobs(jobId, files.map(file => file.path), format.extension)
            .then(items => {
                if (disposed) return;
                validationJobIdRef.current = null;
                setPreflightItems(items);
                setPreflightStatus("ready");
            })
            .catch(error => {
                if (disposed) return;
                validationJobIdRef.current = null;
                setPreflightError(String(error));
                setPreflightStatus("error");
            });

        return () => {
            disposed = true;
            if (validationJobIdRef.current === jobId) {
                validationJobIdRef.current = null;
                void api.cancelJob(jobId).catch(() => undefined);
            }
        };
    }, [preflightKey, preflightNonce]);

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
    const cancelledCount = results.filter(r => !r.success && r.error?.toLowerCase().includes("cancelled")).length;
    const failedCount = results.filter(r => !r.success && !r.error?.toLowerCase().includes("cancelled")).length;
    const progress = files.length > 0 ? (completedCount / files.length) * 100 : 0;
    const preflightReadyCount = preflightItems.filter(item => item.result.is_valid).length;
    const preflightBlockedCount = preflightItems.length - preflightReadyCount;
    const preflightWarningCount = preflightItems.reduce((count, item) => count + item.result.warnings.length, 0);
    const preflightByPath = useMemo(
        () => new Map(preflightItems.map(item => [item.input_path, item.result])),
        [preflightItems],
    );
    const orderedResults = useMemo(() => {
        const fileOrder = new Map(files.map((file, index) => [file.path, index]));
        return [...results].sort(
            (left, right) => (fileOrder.get(left.input_path) ?? Number.MAX_SAFE_INTEGER) -
                (fileOrder.get(right.input_path) ?? Number.MAX_SAFE_INTEGER),
        );
    }, [files, results]);

    const runBatch = async (filesToProcess: FileItem[], suppliedValidation?: BatchValidationItem[]) => {
        if (filesToProcess.length === 0 || runningRef.current) return;
        runningRef.current = true;
        setIsCancelling(false);
        setOperationError(null);
        filesToProcess.forEach(file => onUpdateFileStatus(file.id, "pending"));

        setIsConverting(true);
        isCancelledRef.current = false;
        activeJobIdsRef.current.clear();
        setStartTime(Date.now());

        const validatedQueue: FileItem[] = [];
        const terminalPaths = new Set<string>();
        const validationWarnings = new Map<string, string[]>();

        const cancelledResult = (file: FileItem): ConversionResult => ({
            job_id: crypto.randomUUID(),
            input_path: file.path,
            output_path: "",
            success: false,
            original_size_bytes: file.size,
            converted_size_bytes: 0,
            elapsed_ms: 0,
            error: "Conversion cancelled",
        });

        let validationItems = suppliedValidation;
        if (!validationItems) {
            const validationJobId = crypto.randomUUID();
            activeJobIdsRef.current.add(validationJobId);
            try {
                validationItems = await api.validateJobs(
                    validationJobId,
                    filesToProcess.map(file => file.path),
                    format.extension,
                );
            } catch (error) {
                if (isCancelledRef.current || String(error).toLowerCase().includes("cancelled")) {
                    const cancelled = filesToProcess.map(file => {
                        const result = cancelledResult(file);
                        onUpdateFileStatus(file.id, "cancelled", result);
                        return result;
                    });
                    setResults(prev => [
                        ...prev.filter(result => !cancelled.some(item => item.input_path === result.input_path)),
                        ...cancelled,
                    ]);
                } else {
                    setOperationError(`Batch readiness check failed: ${String(error)}`);
                    filesToProcess.forEach(file => onUpdateFileStatus(
                        file.id,
                        file.result?.error?.toLowerCase().includes("cancelled") ? "cancelled" : "error",
                        file.result,
                    ));
                }
                setIsConverting(false);
                setIsCancelling(false);
                runningRef.current = false;
                activeJobIdsRef.current.delete(validationJobId);
                return;
            }
            activeJobIdsRef.current.delete(validationJobId);
        }

        const validationByPath = new Map(validationItems.map(item => [item.input_path, item.result]));
        for (const file of filesToProcess) {
            const validation = validationByPath.get(file.path);
            if (validation?.is_valid) {
                validatedQueue.push(file);
                validationWarnings.set(file.path, validation.warnings);
                continue;
            }
            const fail: ConversionResult = {
                job_id: crypto.randomUUID(),
                input_path: file.path,
                output_path: "",
                success: false,
                original_size_bytes: file.size,
                converted_size_bytes: 0,
                elapsed_ms: 0,
                error: validation?.error || "Validation pre-flight failed",
            };
            terminalPaths.add(file.path);
            onUpdateFileStatus(file.id, "error", fail);
            setResults(prev => [...prev.filter(result => result.input_path !== file.path), fail]);
        }

        if (validatedQueue.length === 0 || isCancelledRef.current) {
            if (isCancelledRef.current) {
                const cancelled = filesToProcess
                    .filter(file => !terminalPaths.has(file.path))
                    .map(file => {
                        const result = cancelledResult(file);
                        onUpdateFileStatus(file.id, "cancelled", result);
                        return result;
                    });
                setResults(prev => [
                    ...prev.filter(result => !cancelled.some(item => item.input_path === result.input_path)),
                    ...cancelled,
                ]);
            }
            setIsConverting(false);
            runningRef.current = false;
            setIsCancelling(false);
            return;
        }

        const concurrencyLimit = Math.max(1, Math.min(Math.floor(Number(settings.maxParallelJobs)) || 4, 4));
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
                    output_dir: outputDirectory || undefined,
                    target_format: format.extension,
                    crf: settings.crf,
                    resolution: settings.resolution,
                    hardware_accel: settings.hardwareAccel,
                    selected_encoder: settings.selectedEncoder === "auto" ? hardware?.recommended_encoder : settings.selectedEncoder,
                    strip_metadata: settings.stripMetadata,
                    audio_bitrate: settings.audioBitrate,
                    collision_policy: settings.collisionPolicy,
                    rawFile: file.rawFile,
                });
                res.warnings = [...(validationWarnings.get(file.path) || []), ...(res.warnings || [])]
                    .filter((warning, index, all) => all.indexOf(warning) === index);
                if (res.download_url) blobUrlsRef.current.add(res.download_url);
                onUpdateFileStatus(file.id, res.success ? "completed" : res.error?.toLowerCase().includes("cancelled") ? "cancelled" : "error", res);
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
                onUpdateFileStatus(file.id, String(err).toLowerCase().includes("cancelled") ? "cancelled" : "error", fail);
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
        if (isCancelledRef.current) {
            const cancelled = validatedQueue.slice(nextIndex).map(file => {
                const result = cancelledResult(file);
                onUpdateFileStatus(file.id, "cancelled", result);
                return result;
            });
            setResults(prev => [
                ...prev.filter(result => !cancelled.some(item => item.input_path === result.input_path)),
                ...cancelled,
            ]);
        }
        runningRef.current = false;
        setIsCancelling(false);
        setIsConverting(false);
        setActiveWorkerCount(0);
    };

    const handleRun = () => {
        if (runningRef.current) return;
        blobUrlsRef.current.forEach(url => URL.revokeObjectURL(url));
        blobUrlsRef.current.clear();
        setResults([]);
        if (preflightStatus !== "ready") return;
        void runBatch(files, preflightItems);
    };

    const handleCancel = async () => {
        isCancelledRef.current = true;
        setIsCancelling(true);
        const activeIds = Array.from(activeJobIdsRef.current);
        const cancellations = await Promise.allSettled(activeIds.map(id => api.cancelJob(id)));
        const failure = cancellations.find(result => result.status === "rejected");
        if (failure?.status === "rejected") setOperationError("Some jobs already finished or could not be cancelled. Waiting for their final result.");
        // Keep the batch locked until all workers acknowledge completion.
    };

    const handleRetryFailed = () => {
        const failedFiles = files.filter(f => f.status === "error" || f.status === "cancelled");
        if (failedFiles.length > 0) {
            void runBatch(failedFiles);
        }
    };

    const handleShowInFolder = async (filePath: string) => {
        if (!isTauri() || !filePath) return;
        try {
            await api.showInFolder(filePath);
        } catch (err) {
            setOperationError(`Cannot open output folder: ${String(err)}`);
        }
    };

    const handleChooseOutputDirectory = async () => {
        try {
            const selected = await api.chooseOutputDirectory();
            if (selected) setOutputDirectory(selected);
        } catch (error) {
            setOperationError(`Output folder could not be selected: ${String(error)}`);
        }
    };

    const handleCopyReport = async () => {
        const lines = [
            "File2File conversion report",
            `Target: ${format.extension.toUpperCase()}`,
            `Summary: ${successfulCount} successful, ${failedCount} failed, ${cancelledCount} cancelled`,
            `Elapsed: ${(elapsedTime / 1000).toFixed(1)} seconds`,
            "",
            ...orderedResults.map(result => {
                const inputName = result.input_path.split(/[\\/]/).pop() || result.input_path;
                const status = result.success ? "OK" : result.error?.toLowerCase().includes("cancelled") ? "CANCELLED" : "FAILED";
                const details = result.success
                    ? result.output_path
                    : (result.error || "Unknown error").replace(/\s+/g, " ");
                return `[${status}] ${inputName}: ${details}`;
            }),
        ];
        try {
            await navigator.clipboard.writeText(lines.join("\n"));
            setReportCopied(true);
            if (reportCopiedTimerRef.current !== null) {
                window.clearTimeout(reportCopiedTimerRef.current);
            }
            reportCopiedTimerRef.current = window.setTimeout(() => {
                reportCopiedTimerRef.current = null;
                setReportCopied(false);
            }, 2000);
        } catch (error) {
            setOperationError(`Conversion report could not be copied: ${String(error)}`);
        }
    };

    const done = completedCount === files.length && !isConverting && files.length > 0;

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
            {operationError && <p role="alert" className="text-error">{operationError}</p>}
            {isCancelling && <p role="status">Cancelling — waiting for active jobs to stop…</p>}
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
                                disabled={isCancelling}
                                className="btn-secondary h-[32px] px-3 text-[12px] text-error border-error/30 hover:bg-error/10 flex items-center gap-1.5 disabled:opacity-50"
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

                            <div className="flex items-center justify-between gap-4 px-4 py-3 bg-surface border border-border rounded-lg">
                                <div className="min-w-0">
                                    <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Batch readiness</div>
                                    {preflightStatus === "loading" && (
                                        <div className="text-[12px] text-text-secondary flex items-center gap-2 mt-1">
                                            <Loader2 className="w-3.5 h-3.5 animate-spin text-accent" />
                                            Inspecting {files.length} input{files.length === 1 ? "" : "s"}…
                                        </div>
                                    )}
                                    {preflightStatus === "ready" && (
                                        <div className="text-[12px] text-text-secondary mt-1">
                                            <span className="text-success">{preflightReadyCount} ready</span>
                                            {preflightBlockedCount > 0 && <span className="text-error"> · {preflightBlockedCount} blocked</span>}
                                            {preflightWarningCount > 0 && <span className="text-warning"> · {preflightWarningCount} warning{preflightWarningCount === 1 ? "" : "s"}</span>}
                                        </div>
                                    )}
                                    {preflightStatus === "error" && (
                                        <div className="text-[12px] text-error mt-1 truncate" title={preflightError || undefined}>
                                            Readiness check failed: {preflightError}
                                        </div>
                                    )}
                                </div>
                                {preflightStatus === "error" && (
                                    <button onClick={() => setPreflightNonce(value => value + 1)} className="btn-secondary h-[34px] flex items-center gap-2 shrink-0">
                                        <RefreshCw className="w-3.5 h-3.5" />
                                        Check again
                                    </button>
                                )}
                            </div>

                            {isTauri() && (
                                <div className="flex items-center justify-between gap-4 px-4 py-3 bg-surface border border-border rounded-lg">
                                    <div className="min-w-0">
                                        <div className="text-tiny font-bold text-text-muted uppercase tracking-widest">Output location</div>
                                        <div className="text-[12px] text-text-secondary truncate" title={outputDirectory || undefined}>
                                            {outputDirectory || "Alongside each source file"}
                                        </div>
                                    </div>
                                    <div className="flex items-center gap-1 shrink-0">
                                        {outputDirectory && (
                                            <button onClick={() => setOutputDirectory(null)} className="p-2 text-text-muted hover:text-text-primary" aria-label="Use source folders">
                                                <X className="w-4 h-4" />
                                            </button>
                                        )}
                                        <button onClick={handleChooseOutputDirectory} className="btn-secondary h-[34px] flex items-center gap-2">
                                            <FolderOpen className="w-4 h-4" />
                                            Choose folder
                                        </button>
                                    </div>
                                </div>
                            )}

                            {/* Execution Area */}
                            <div className="flex-1 flex flex-col items-center justify-center border border-border bg-surface-raised rounded-lg p-8">
                                {!isConverting ? (
                                    <div className="text-center space-y-6 max-w-sm">
                                        <div className="space-y-1.5">
                                            <h3 className="text-[17px] font-semibold text-text-primary">
                                                {preflightStatus === "loading" ? "Checking inputs" : preflightReadyCount > 0 ? "Ready to convert" : "No convertible inputs"}
                                            </h3>
                                            <p className="text-[13px] text-text-muted">
                                                {preflightStatus === "ready"
                                                    ? `${preflightReadyCount} of ${files.length} files can be processed locally.`
                                                    : "File contents and required streams are checked before work begins."}
                                            </p>
                                        </div>
                                        <button
                                            onClick={handleRun}
                                            disabled={preflightStatus !== "ready" || preflightReadyCount === 0}
                                            className="btn-primary w-full h-[48px] text-[15px] font-semibold shadow-sm"
                                        >
                                            {preflightStatus === "loading" ? "Checking readiness…" : `Convert ${preflightReadyCount} file${preflightReadyCount === 1 ? "" : "s"}`}
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
                                    <h2 className="text-[22px] font-semibold text-text-primary tracking-tight">
                                        {cancelledCount > 0 ? "Batch stopped" : "Conversion complete"}
                                    </h2>
                                    <p className="text-[13px] text-text-secondary">
                                        {successfulCount} successful · {failedCount} failed{cancelledCount > 0 ? ` · ${cancelledCount} cancelled` : ""} · {(elapsedTime / 1000).toFixed(1)}s total
                                    </p>
                                </div>
                                <div className="flex gap-2">
                                    <button onClick={handleCopyReport} className="btn-secondary flex items-center gap-1.5">
                                        <ClipboardCopy className="w-3.5 h-3.5" />
                                        <span>{reportCopied ? "Copied" : "Copy report"}</span>
                                    </button>
                                    {failedCount + cancelledCount > 0 && (
                                        <button
                                            onClick={handleRetryFailed}
                                            className="btn-secondary flex items-center gap-1.5 text-warning"
                                        >
                                            <RotateCcw className="w-3.5 h-3.5" />
                                            <span>Retry unfinished</span>
                                        </button>
                                    )}
                                    <button onClick={onResetToUpload} className="btn-secondary">New conversion</button>
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
                                {orderedResults.map((res) => (
                                    <div key={res.job_id} className="min-h-[60px] flex items-center justify-between px-4 py-3 bg-surface border border-border rounded-lg group">
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
                                                    <>
                                                        <div className="text-tiny font-mono text-text-muted uppercase">
                                                            {formatBytes(res.original_size_bytes)} → {formatBytes(res.converted_size_bytes)} · {res.elapsed_ms}ms
                                                        </div>
                                                        {res.warnings && res.warnings.length > 0 && (
                                                            <div className="text-[10px] text-warning truncate max-w-md" title={res.warnings.join("\n")}>{res.warnings.join(" · ")}</div>
                                                        )}
                                                    </>
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
                                                <span className={`text-tiny font-mono px-2 py-1 rounded ${res.error?.toLowerCase().includes("cancelled") ? "text-warning bg-warning/10" : "text-error bg-error/10"}`}>
                                                    {res.error?.toLowerCase().includes("cancelled") ? "Cancelled" : "Failed"}
                                                </span>
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
                        {files.map(f => {
                            const readiness = preflightByPath.get(f.path);
                            const pendingLabel = preflightStatus === "loading"
                                ? "checking"
                                : readiness?.is_valid
                                    ? readiness.warnings.length > 0 ? "warning" : "ready"
                                    : "blocked";
                            const displayStatus = f.status === "pending" && !isConverting ? pendingLabel : f.status;
                            return <div key={f.id} className="px-4 py-3 flex items-center justify-between transition-colors" title={readiness?.error || readiness?.warnings.join("\n") || undefined}>
                                <div className="flex items-center gap-2.5 truncate">
                                    <div className={`w-1.5 h-1.5 rounded-full ${
                                        f.status === "completed" ? "bg-success" :
                                        f.status === "converting" ? "bg-accent animate-pulse" :
                                        f.status === "error" ? "bg-error" :
                                        f.status === "cancelled" ? "bg-warning" :
                                        displayStatus === "ready" ? "bg-success" :
                                        displayStatus === "warning" ? "bg-warning" :
                                        displayStatus === "blocked" ? "bg-error" :
                                        "bg-text-disabled"
                                    }`} />
                                    <div className="min-w-0">
                                        <div className={`text-[12px] truncate ${f.status === "pending" ? "text-text-muted" : "text-text-primary"}`}>{f.name}</div>
                                        {readiness?.file_info && !isConverting && (
                                            <div className="text-[10px] text-text-disabled font-mono">
                                                {readiness.file_info.width > 0 ? `${readiness.file_info.width}×${readiness.file_info.height} · ` : ""}
                                                {readiness.file_info.duration > 0 ? `${readiness.file_info.duration.toFixed(1)}s` : readiness.file_info.format_name}
                                            </div>
                                        )}
                                    </div>
                                </div>
                                <span className={`text-tiny font-mono uppercase ${
                                    f.status === "completed" ? "text-success" :
                                    f.status === "converting" ? "text-accent font-semibold" :
                                    f.status === "error" ? "text-error" :
                                    displayStatus === "ready" ? "text-success" :
                                    displayStatus === "warning" ? "text-warning" :
                                    displayStatus === "blocked" ? "text-error" :
                                    "text-text-disabled"
                                }`}>
                                    {displayStatus}
                                </span>
                            </div>;
                        })}
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
