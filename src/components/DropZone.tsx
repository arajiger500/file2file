import React, { useRef, useState } from "react";
import { Upload, Folder, File as FileIcon, X, Loader2 } from "lucide-react";
import { FileItem, FileCategory } from "../types";

interface DropZoneProps {
    files: FileItem[];
    onAddFiles: (newFiles: FileItem[]) => void;
    onRemoveFile: (id: string) => void;
    onClearFiles: () => void;
}

const detectCategory = (ext: string): FileCategory => {
    const e = ext.toLowerCase();
    if (["mp4", "mkv", "mov", "avi", "webm", "flv", "wmv", "m4v", "ts", "3gp", "ogv", "vob", "gif"].includes(e)) return "video";
    if (["mp3", "wav", "flac", "aac", "ogg", "m4a", "opus", "wma", "aiff"].includes(e)) return "audio";
    if (["png", "jpg", "jpeg", "webp", "gif", "bmp", "avif", "tiff", "ico", "heic", "tga", "psd"].includes(e)) return "image";
    if (["pdf", "docx", "doc", "md", "html", "txt", "epub", "rtf", "odt"].includes(e)) return "document";
    if (["svg", "eps", "ai"].includes(e)) return "vector";
    return "unknown";
};

export const DropZone: React.FC<DropZoneProps> = ({ files, onAddFiles, onRemoveFile, onClearFiles }) => {
    const [isDragging, setIsDragging] = useState(false);
    const fileRef = useRef<HTMLInputElement>(null);

    const processFiles = (raw: FileList | null) => {
        if (!raw) return;
        const items = Array.from(raw).map(f => ({
            id: Math.random().toString(36).substring(2),
            path: (f as any).path || f.name,
            name: f.name,
            size: f.size,
            extension: f.name.split(".").pop() || "",
            category: detectCategory(f.name.split(".").pop() || ""),
            status: "pending" as const,
            rawFile: f
        }));
        onAddFiles(items);
    };

    return (
        <div className="space-y-6">
            <div
                onDragOver={(e) => { e.preventDefault(); setIsDragging(true); }}
                onDragLeave={() => setIsDragging(false)}
                onDrop={(e) => { e.preventDefault(); setIsDragging(false); processFiles(e.dataTransfer.files); }}
                onClick={() => fileRef.current?.click()}
                className={`group relative overflow-hidden h-72 border-2 border-dashed rounded-[2rem] flex flex-col items-center justify-center transition-all duration-500 cursor-pointer ${
                    isDragging ? "border-blue-500 bg-blue-500/5 scale-[0.98]" : "border-zinc-800 bg-zinc-900/30 hover:bg-zinc-900/50 hover:border-zinc-700"
                }`}
            >
                <div className="absolute inset-0 bg-gradient-to-t from-blue-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity" />

                <input type="file" ref={fileRef} multiple className="hidden" onChange={(e) => processFiles(e.target.files)} />

                <div className="relative z-10 space-y-4 text-center">
                    <div className="w-20 h-20 mx-auto rounded-3xl bg-zinc-900 border border-zinc-800 flex items-center justify-center shadow-2xl group-hover:scale-110 group-hover:rotate-3 transition-transform duration-500">
                        <Upload className="w-8 h-8 text-blue-400" />
                    </div>
                    <div>
                        <h3 className="text-xl font-bold text-white">Import Media</h3>
                        <p className="text-zinc-500 text-sm mt-1">Drag files or folder to begin processing</p>
                    </div>
                </div>
            </div>

            {files.length > 0 && (
                <div className="glass-card p-6 border-zinc-800/50">
                    <div className="flex items-center justify-between mb-4">
                        <h4 className="text-xs font-bold uppercase tracking-widest text-zinc-500 flex items-center gap-2">
                            <Folder className="w-3.5 h-3.5" />
                            Batch Queue ({files.length})
                        </h4>
                        <button onClick={onClearFiles} className="text-[10px] font-bold text-zinc-600 hover:text-red-400 transition-colors uppercase">Clear All</button>
                    </div>
                    <div className="grid grid-cols-1 gap-2 max-h-80 overflow-y-auto pr-2">
                        {files.map(f => (
                            <div key={f.id} className="group/item flex items-center justify-between p-4 rounded-2xl bg-zinc-900/50 border border-zinc-800 hover:border-zinc-600 transition-all">
                                <div className="flex items-center gap-4 truncate">
                                    <div className="p-2.5 rounded-xl bg-zinc-800 group-hover/item:bg-blue-500/10 transition-colors">
                                        <FileIcon className="w-4 h-4 text-zinc-400 group-hover/item:text-blue-400" />
                                    </div>
                                    <div className="truncate">
                                        <div className="text-sm font-semibold text-zinc-200 truncate">{f.name}</div>
                                        <div className="text-[10px] text-zinc-500 uppercase font-mono mt-0.5">{(f.size / (1024*1024)).toFixed(2)} MB • {f.extension}</div>
                                    </div>
                                </div>
                                <button onClick={(e) => { e.stopPropagation(); onRemoveFile(f.id); }} className="opacity-0 group-hover/item:opacity-100 p-2 hover:bg-red-500/10 rounded-lg text-zinc-600 hover:text-red-400 transition-all">
                                    <X className="w-4 h-4" />
                                </button>
                            </div>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
};
