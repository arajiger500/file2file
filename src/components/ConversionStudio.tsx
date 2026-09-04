import React, { useState, useMemo } from "react";
import { ArrowLeft, Play, Loader2, CheckCircle2, Cpu, Zap, HardDrive, Clock, RefreshCw, Download, FolderCheck, AlertCircle } from "lucide-react";
import { FileItem, FormatOption, AdvancedSettings, ConversionResult, HardwareInfo } from "../types";
import { api, downloadFile, isTauri } from "../services/api";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

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
    const [isExportingAll, setIsExportingAll] = useState(false);

    const completedCount = files.filter(f => f.status === "completed" || f.status === "error").length;
    const progress = files.length > 0 ? (completedCount / files.length) * 100 : 0;

    const handleRun = async () => {
        setIsConverting(true);
        setResults([]);
        setStartTime(Date.now());

        const fileQueue = [...files];
        const validatedQueue: FileItem[] = [];

        // Step 1: Intelligent Validation & Pre-flight checks
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
                validatedQueue.push(file); // Fallback to attempt conversion if validation service fails
            }
        }

        if (validatedQueue.length === 0 && fileQueue.length > 0) {
            setIsConverting(false);
            setCurrentTime(Date.now());
            return;
        }

        // Concurrency limit: Use CPU cores or fallback to 4
        const concurrencyLimit = hardware?.cpu_cores || 4;
        let activeIndex = 0;

        const processFile = async (file: FileItem): Promise<ConversionResult> => {
// ...
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
        setIsExportingAll(true);
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
                        // Using a simple path join logic for the command
                        const dest = `${selectedDir}/${filename}`;
                        // We use the copy_file command from our API
                        await (await import("@tauri-apps/api/core")).invoke("copy_file", { src: res.output_path, dest });
                    }
                }
                // Optional: Open the folder after export
                await (await import("@tauri-apps/plugin-shell")).open(selectedDir);
            }
        } catch (err) {
            console.error("Export all error:", err);
        } finally {
            setIsExportingAll(false);
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
        <div className="max-w-5xl mx-auto space-y-8 pb-32 animate-in fade-in duration-700">
            <div className="flex items-center justify-between">
                <button onClick={onBackToFormats} className="group flex items-center gap-3 text-zinc-500 hover:text-white transition-all">
                    <div className="p-2 rounded-xl bg-zinc-900 border border-zinc-800 group-hover:border-zinc-700 transition-colors">
                        <ArrowLeft className="w-4 h-4" />
                    </div>
                    <span className="text-sm font-bold uppercase tracking-wider">Configuration</span>
                </button>

                {isConverting && (
                    <div className="flex items-center gap-4 px-4 py-2 rounded-2xl bg-blue-500/10 border border-blue-500/20">
                        <Loader2 className="w-4 h-4 text-blue-400 animate-spin" />
                        <span className="text-xs font-bold text-blue-400 uppercase tracking-widest">
                            Processing {completedCount}/{files.length}
                        </span>
                    </div>
                )}
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-12 gap-8">
                <div className="lg:col-span-8 space-y-6">
                    {/* Pipeline Info Card */}
                    <div className="glass-card p-10 relative overflow-hidden bg-zinc-900/40 border-zinc-800">
                        <div className={cn(
                            "absolute top-0 left-0 h-1 bg-gradient-to-r from-blue-600 via-purple-600 to-blue-600 transition-all duration-500",
                            isConverting ? "w-full animate-pulse" : "w-full opacity-50"
                        )} />

                        <div className="space-y-6">
                            <div className="flex items-center gap-3">
                                <div className="px-3 py-1 rounded-lg bg-blue-500/10 border border-blue-500/20 text-blue-400 font-mono text-[10px] font-bold tracking-widest uppercase">Target Pipeline</div>
                                <div className="h-px flex-1 bg-zinc-800" />
                            </div>

                            <div className="flex flex-col md:flex-row items-start md:items-end justify-between gap-6">
                                <div>
                                    <h1 className="text-4xl font-black text-white tracking-tighter uppercase">{format.name}</h1>
                                    <p className="text-zinc-500 mt-2 font-medium">{format.description}</p>
                                </div>
                                <div className="flex items-center gap-2">
                                    <span className="px-4 py-2 rounded-xl bg-zinc-950 border border-zinc-800 text-zinc-400 font-mono text-xs font-bold uppercase">.{format.extension}</span>
                                </div>
                            </div>

                            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 pt-4">
                                <div className="p-4 rounded-2xl bg-zinc-950/50 border border-zinc-800/50 group hover:border-blue-500/30 transition-colors">
                                    <div className="text-[10px] font-bold text-zinc-600 uppercase mb-2 group-hover:text-blue-400 transition-colors">Engine Core</div>
                                    <div className="flex items-center gap-2 text-zinc-300 font-bold text-sm">
                                        <Cpu className="w-4 h-4 text-blue-500" />
                                        {format.sidecar_engine}
                                    </div>
                                </div>
                                <div className="p-4 rounded-2xl bg-zinc-950/50 border border-zinc-800/50 group hover:border-amber-500/30 transition-colors">
                                    <div className="text-[10px] font-bold text-zinc-600 uppercase mb-2 group-hover:text-amber-400 transition-colors">Processing</div>
                                    <div className="flex items-center gap-2 text-zinc-300 font-bold text-sm">
                                        <Zap className="w-4 h-4 text-amber-500" />
                                        {settings.hardwareAccel ? "Hardware Accelerated" : "Software Core"}
                                    </div>
                                </div>
                                <div className="p-4 rounded-2xl bg-zinc-950/50 border border-zinc-800/50 group hover:border-emerald-500/30 transition-colors">
                                    <div className="text-[10px] font-bold text-zinc-600 uppercase mb-2 group-hover:text-emerald-400 transition-colors">Quality Profile</div>
                                    <div className="flex items-center gap-2 text-zinc-300 font-bold text-sm">
                                        <CheckCircle2 className="w-4 h-4 text-emerald-500" />
                                        {format.is_lossless ? "Lossless" : "Balanced (CRF 23)"}
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Progress Bar (Visible only when converting or done) */}
                    {(isConverting || done) && (
                        <div className="space-y-3 animate-in fade-in slide-in-from-top-4 duration-500">
                            <div className="flex items-center justify-between text-[10px] font-bold uppercase tracking-widest">
                                <span className="text-zinc-500">{isConverting ? "Processing Batch..." : "Conversion Complete"}</span>
                                <span className="text-white">{Math.round(progress)}%</span>
                            </div>
                            <div className="h-3 w-full bg-zinc-900 rounded-full overflow-hidden border border-zinc-800 p-0.5">
                                <div
                                    className="h-full bg-gradient-to-r from-blue-600 to-purple-600 rounded-full transition-all duration-500 ease-out"
                                    style={{ width: `${progress}%` }}
                                />
                            </div>
                        </div>
                    )}

                    {!done ? (
                        <button
                            onClick={handleRun}
                            disabled={isConverting}
                            className={cn(
                                "w-full group p-8 rounded-[2.5rem] font-black text-2xl flex flex-col items-center justify-center gap-2 transition-all duration-500 shadow-2xl relative overflow-hidden",
                                isConverting
                                    ? "bg-zinc-900 text-zinc-600 cursor-not-allowed scale-[0.98]"
                                    : "bg-white text-black hover:scale-[1.02] hover:bg-zinc-200 active:scale-95"
                            )}
                        >
                            {isConverting ? (
                                <>
                                    <Loader2 className="w-10 h-10 animate-spin mb-2" />
                                    <span className="uppercase tracking-tighter italic font-black text-xl">Transcoding in Parallel</span>
                                    <span className="text-[10px] font-bold opacity-60 uppercase tracking-[0.2em]">{hardware?.cpu_cores || 4} Parallel Workers Active</span>
                                </>
                            ) : (
                                <>
                                    <div className="flex items-center gap-4">
                                        <Play className="w-8 h-8 fill-current" />
                                        <span className="uppercase tracking-tighter text-3xl">Initiate Batch Export</span>
                                    </div>
                                    <span className="text-[10px] font-bold opacity-60 uppercase tracking-[0.3em]">Hardware Accelerated Pipeline</span>
                                </>
                            )}
                        </button>
                    ) : (
                        <div className="animate-in fade-in slide-in-from-bottom-6 duration-700">
                            <div className="glass-card p-8 border-emerald-500/20 bg-emerald-500/5 space-y-8">
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center gap-4">
                                        <div className="w-14 h-14 rounded-2xl bg-emerald-500 flex items-center justify-center shadow-lg shadow-emerald-500/40">
                                            <CheckCircle2 className="w-8 h-8 text-white" />
                                        </div>
                                        <div>
                                            <h2 className="text-3xl font-black text-white uppercase tracking-tighter">Job Finalized</h2>
                                            <div className="flex items-center gap-3 mt-1">
                                                <span className="text-emerald-400 font-bold text-xs uppercase tracking-widest">{results.filter(r => r.success).length} Successful</span>
                                                <div className="w-1 h-1 rounded-full bg-zinc-700" />
                                                <span className="text-zinc-500 font-bold text-xs uppercase tracking-widest">{totalElapsed}ms Total</span>
                                            </div>
                                        </div>
                                    </div>

                                    {totalSavedBytes > 0 && (
                                        <div className="text-right">
                                            <div className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest">Storage Saved</div>
                                            <div className="text-xl font-black text-emerald-400">{(totalSavedBytes / (1024*1024)).toFixed(1)} MB</div>
                                        </div>
                                    )}
                                </div>

                                <div className="space-y-3 max-h-[400px] overflow-y-auto pr-2 custom-scrollbar">
                                    {results.map((res, i) => (
                                        <div key={i} className="flex items-center justify-between p-5 rounded-3xl bg-zinc-950 border border-zinc-800 hover:border-zinc-700 transition-colors group">
                                            <div className="flex items-center gap-4 truncate">
                                                <div className={cn(
                                                    "p-3 rounded-2xl border font-mono text-[10px] font-bold uppercase tracking-tighter",
                                                    res.success ? "bg-zinc-900 border-zinc-800 text-zinc-400" : "bg-red-500/10 border-red-500/20 text-red-500"
                                                )}>
                                                    {res.success ? "Final" : "Fail"}
                                                </div>
                                                <div className="truncate">
                                                    <div className="text-sm font-bold text-white truncate uppercase tracking-tight">{res.output_path.split(/[\\/]/).pop()}</div>
                                                    <div className="flex items-center gap-3 text-[10px] font-bold text-zinc-500 uppercase mt-1">
                                                        {res.success ? (
                                                            <>
                                                                <span className="flex items-center gap-1"><HardDrive className="w-3 h-3" /> {(res.converted_size_bytes / (1024*1024)).toFixed(2)} MB</span>
                                                                <span className="flex items-center gap-1"><Clock className="w-3 h-3" /> {res.elapsed_ms}ms</span>
                                                            </>
                                                        ) : (
                                                            <span className="text-red-500/80 flex items-center gap-1"><AlertCircle className="w-3 h-3" /> Error in pipeline</span>
                                                        )}
                                                    </div>
                                                </div>
                                            </div>
                                            {res.success ? (
                                                <button onClick={() => downloadFile(res)} className="px-5 py-2.5 rounded-xl bg-zinc-900 border border-zinc-800 text-white font-bold text-[10px] uppercase tracking-widest hover:bg-white hover:text-black transition-all shadow-lg active:scale-95 flex items-center gap-2">
                                                    <Download className="w-3.5 h-3.5" /> Save
                                                </button>
                                            ) : (
                                                <div className="group-hover:block hidden max-w-[200px] truncate text-[9px] text-red-400 bg-red-400/5 px-2 py-1 rounded-md border border-red-400/10">
                                                    {res.error}
                                                </div>
                                            )}
                                        </div>
                                    ))}
                                </div>

                                <div className="pt-4 flex items-center gap-4">
                                    {isTauri() && results.some(r => r.success) && (
                                        <button
                                            onClick={handleExportAll}
                                            disabled={isExportingAll}
                                            className="flex-[2] px-6 py-5 rounded-2xl bg-white text-black font-black text-sm uppercase tracking-[0.2em] hover:bg-zinc-200 transition-all flex items-center justify-center gap-3 shadow-xl active:scale-95 disabled:opacity-50"
                                        >
                                            {isExportingAll ? <Loader2 className="w-5 h-5 animate-spin" /> : <FolderCheck className="w-5 h-5" />}
                                            Export All to Folder
                                        </button>
                                    )}
                                    <button onClick={onResetToUpload} className="flex-1 px-6 py-5 rounded-2xl bg-zinc-900 border border-zinc-800 text-zinc-400 font-bold text-xs uppercase tracking-widest hover:text-white hover:border-zinc-600 transition-all flex items-center justify-center gap-2">
                                        <RefreshCw className="w-4 h-4" /> New Job
                                    </button>
                                </div>
                            </div>
                        </div>
                    )}
                </div>

                {/* Right Sidebar - Queue Status */}
                <div className="lg:col-span-4 space-y-6">
                    <div className="glass-card p-6 border-zinc-800/50 flex flex-col h-full bg-zinc-950/20">
                        <div className="flex items-center justify-between mb-6">
                            <h4 className="text-[10px] font-bold uppercase tracking-[0.2em] text-zinc-500">Resource Monitor</h4>
                            {isConverting && <div className="text-[9px] font-bold text-blue-400 animate-pulse uppercase tracking-widest">Active</div>}
                        </div>

                        <div className="space-y-2 overflow-y-auto max-h-[600px] custom-scrollbar pr-2">
                            {files.map(f => (
                                <div key={f.id} className={cn(
                                    "flex items-center justify-between p-3 rounded-2xl border transition-all duration-300",
                                    f.status === "converting" ? "bg-blue-500/5 border-blue-500/30 scale-[1.02]" :
                                    f.status === "completed" ? "bg-emerald-500/5 border-emerald-500/20 opacity-60" :
                                    f.status === "error" ? "bg-red-500/5 border-red-500/20" :
                                    "bg-zinc-950/50 border-zinc-900"
                                )}>
                                    <div className="flex items-center gap-3 truncate">
                                        <div className={cn(
                                            "w-2 h-2 rounded-full",
                                            f.status === "completed" ? "bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]" :
                                            f.status === "converting" ? "bg-blue-500 animate-pulse shadow-[0_0_8px_rgba(59,130,246,0.5)]" :
                                            f.status === "error" ? "bg-red-500" :
                                            "bg-zinc-800"
                                        )} />
                                        <span className={cn(
                                            "text-[11px] font-bold truncate uppercase tracking-tighter",
                                            f.status === "converting" ? "text-blue-400" : "text-zinc-400"
                                        )}>{f.name}</span>
                                    </div>
                                    {f.status === "converting" && <Loader2 className="w-3 h-3 text-blue-500 animate-spin flex-shrink-0" />}
                                    {f.status === "completed" && <CheckCircle2 className="w-3 h-3 text-emerald-500 flex-shrink-0" />}
                                </div>
                            ))}
                        </div>

                        {isConverting && (
                            <div className="mt-8 p-4 rounded-2xl bg-zinc-900/50 border border-zinc-800 space-y-3">
                                <div className="text-[9px] font-bold text-zinc-600 uppercase tracking-widest">Throughput Engine</div>
                                <div className="flex items-center justify-between">
                                    <span className="text-[10px] font-bold text-zinc-400">Concurrency</span>
                                    <span className="text-[10px] font-mono text-white">{hardware?.cpu_cores || 4}x</span>
                                </div>
                                <div className="h-1 w-full bg-zinc-800 rounded-full overflow-hidden">
                                    <div className="h-full bg-blue-500 animate-progress-indefinite" />
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
};
