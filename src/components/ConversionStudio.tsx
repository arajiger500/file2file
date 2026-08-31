import React, { useState } from "react";
import { ArrowLeft, Play, Loader2, CheckCircle2, Cpu, Zap, HardDrive, Clock, RefreshCw } from "lucide-react";
import { FileItem, FormatOption, AdvancedSettings, ConversionResult } from "../types";
import { api, downloadFile } from "../services/api";

interface ConversionStudioProps {
    files: FileItem[];
    format: FormatOption;
    settings: AdvancedSettings;
    onBackToFormats: () => void;
    onResetToUpload: () => void;
    onUpdateFileStatus: (id: string, status: FileItem["status"], result?: ConversionResult) => void;
}

export const ConversionStudio: React.FC<ConversionStudioProps> = ({ files, format, settings, onBackToFormats, onResetToUpload, onUpdateFileStatus }) => {
    const [isConverting, setIsConverting] = useState(false);
    const [results, setResults] = useState<ConversionResult[]>([]);

    const handleRun = async () => {
        setIsConverting(true);
        const newResults: ConversionResult[] = [];
        for (const file of files) {
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
                newResults.push(res);
            } catch (err) {
                const fail: ConversionResult = { job_id: Math.random().toString(), input_path: file.path, output_path: file.path, success: false, original_size_bytes: file.size, converted_size_bytes: 0, elapsed_ms: 0, error: String(err) };
                onUpdateFileStatus(file.id, "error", fail);
                newResults.push(fail);
            }
        }
        setResults(newResults);
        setIsConverting(false);
    };

    const done = results.length > 0 && !isConverting;

    return (
        <div className="max-w-5xl mx-auto space-y-8 pb-32">
            <div className="flex items-center justify-between">
                <button onClick={onBackToFormats} className="group flex items-center gap-3 text-zinc-500 hover:text-white transition-all">
                    <div className="p-2 rounded-xl bg-zinc-900 border border-zinc-800 group-hover:border-zinc-700 transition-colors">
                        <ArrowLeft className="w-4 h-4" />
                    </div>
                    <span className="text-sm font-bold uppercase tracking-wider">Configuration</span>
                </button>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-12 gap-8">
                <div className="lg:col-span-8 space-y-6">
                    <div className="glass-card p-10 relative overflow-hidden bg-zinc-900/40">
                        <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-blue-600 via-purple-600 to-blue-600 opacity-50" />

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
                                <div className="p-4 rounded-2xl bg-zinc-950/50 border border-zinc-800/50">
                                    <div className="text-[10px] font-bold text-zinc-600 uppercase mb-2">Engine Core</div>
                                    <div className="flex items-center gap-2 text-zinc-300 font-bold text-sm">
                                        <Cpu className="w-4 h-4 text-blue-500" />
                                        {format.sidecar_engine}
                                    </div>
                                </div>
                                <div className="p-4 rounded-2xl bg-zinc-950/50 border border-zinc-800/50">
                                    <div className="text-[10px] font-bold text-zinc-600 uppercase mb-2">Processing</div>
                                    <div className="flex items-center gap-2 text-zinc-300 font-bold text-sm">
                                        <Zap className="w-4 h-4 text-amber-500" />
                                        {settings.hardwareAccel ? "Hardware" : "Software"}
                                    </div>
                                </div>
                                <div className="p-4 rounded-2xl bg-zinc-950/50 border border-zinc-800/50">
                                    <div className="text-[10px] font-bold text-zinc-600 uppercase mb-2">Quality</div>
                                    <div className="flex items-center gap-2 text-zinc-300 font-bold text-sm">
                                        <CheckCircle2 className="w-4 h-4 text-emerald-500" />
                                        {format.is_lossless ? "Lossless" : "High (CRF 23)"}
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    {!done ? (
                        <button
                            onClick={handleRun}
                            disabled={isConverting}
                            className={`w-full group p-6 rounded-[2rem] font-black text-xl flex items-center justify-center gap-4 transition-all duration-500 shadow-2xl relative overflow-hidden ${
                                isConverting ? "bg-zinc-900 text-zinc-600 cursor-not-allowed" : "bg-white text-black hover:scale-[1.02] hover:bg-zinc-200 active:scale-95"
                            }`}
                        >
                            {isConverting ? (
                                <>
                                    <Loader2 className="w-8 h-8 animate-spin" />
                                    <span className="uppercase tracking-tighter italic">Transcoding Media...</span>
                                </>
                            ) : (
                                <>
                                    <Play className="w-6 h-6 fill-current" />
                                    <span className="uppercase tracking-tighter">Initiate Universal Export</span>
                                </>
                            )}
                        </button>
                    ) : (
                        <div className="animate-in fade-in slide-in-from-bottom-6 duration-700">
                            <div className="glass-card p-8 border-emerald-500/20 bg-emerald-500/5 space-y-6">
                                <div className="flex items-center gap-4">
                                    <div className="w-12 h-12 rounded-2xl bg-emerald-500 flex items-center justify-center shadow-lg shadow-emerald-500/40">
                                        <CheckCircle2 className="w-7 h-7 text-white" />
                                    </div>
                                    <div>
                                        <h2 className="text-2xl font-black text-white uppercase tracking-tighter">Job Finalized</h2>
                                        <p className="text-emerald-400/80 font-bold text-xs uppercase tracking-widest mt-1">Successfully processed {results.filter(r => r.success).length} items</p>
                                    </div>
                                </div>

                                <div className="space-y-3">
                                    {results.map((res, i) => (
                                        <div key={i} className="flex items-center justify-between p-5 rounded-3xl bg-zinc-950 border border-zinc-800">
                                            <div className="flex items-center gap-4 truncate">
                                                <div className="p-3 rounded-2xl bg-zinc-900 border border-zinc-800 text-zinc-400 font-mono text-[10px] font-bold uppercase tracking-tighter">Final</div>
                                                <div className="truncate">
                                                    <div className="text-sm font-bold text-white truncate uppercase tracking-tight">{res.output_path.split("/").pop()}</div>
                                                    <div className="flex items-center gap-3 text-[10px] font-bold text-zinc-500 uppercase mt-1">
                                                        <span className="flex items-center gap-1"><HardDrive className="w-3 h-3" /> {(res.converted_size_bytes / (1024*1024)).toFixed(2)} MB</span>
                                                        <span className="flex items-center gap-1"><Clock className="w-3 h-3" /> {res.elapsed_ms}ms</span>
                                                    </div>
                                                </div>
                                            </div>
                                            {res.success ? (
                                                <button onClick={() => downloadFile(res)} className="px-6 py-3 rounded-2xl bg-white text-black font-black text-xs uppercase tracking-widest hover:bg-zinc-200 transition-colors shadow-lg active:scale-95">Download</button>
                                            ) : (
                                                <span className="text-[10px] font-bold text-red-500 uppercase">Error: {res.error}</span>
                                            )}
                                        </div>
                                    ))}
                                </div>

                                <div className="pt-4 flex items-center gap-4">
                                    <button onClick={onResetToUpload} className="flex-1 px-6 py-4 rounded-2xl bg-zinc-900 border border-zinc-800 text-zinc-300 font-bold text-xs uppercase tracking-widest hover:text-white hover:border-zinc-600 transition-all flex items-center justify-center gap-2">
                                        <RefreshCw className="w-4 h-4" /> New Export
                                    </button>
                                </div>
                            </div>
                        </div>
                    )}
                </div>

                <div className="lg:col-span-4 space-y-6">
                    <div className="glass-card p-6 border-zinc-800/50">
                        <h4 className="text-[10px] font-bold uppercase tracking-[0.2em] text-zinc-500 mb-4">Export Queue</h4>
                        <div className="space-y-2">
                            {files.map(f => (
                                <div key={f.id} className="flex items-center gap-3 p-3 rounded-xl bg-zinc-950/50 border border-zinc-800">
                                    <div className={`w-1.5 h-1.5 rounded-full ${f.status === "completed" ? "bg-emerald-500" : f.status === "converting" ? "bg-blue-500 animate-pulse" : "bg-zinc-700"}`} />
                                    <span className="text-[11px] font-bold text-zinc-400 truncate uppercase tracking-tighter">{f.name}</span>
                                </div>
                            ))}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};
