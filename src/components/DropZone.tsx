import React, { useEffect, useRef, useState } from "react";
import { Upload, File as FileIcon, X, Plus, Trash2 } from "lucide-react";
import { FileItem, FileCategory, FormatOption } from "../types";
import { api, isTauri } from "../services/api";

interface DropZoneProps {
    files: FileItem[];
    onAddFiles: (newFiles: FileItem[]) => void;
    onRemoveFile: (id: string) => void;
    onClearFiles: () => void;
    onDirectConvert?: (format: FormatOption) => void;
}

const detectCategory = (ext: string): FileCategory => {
    const e = ext.toLowerCase();

    if (["mp4", "mkv", "mov", "avi", "webm", "flv", "wmv", "m4v", "ts", "3gp", "ogv", "vob", "gif"].includes(e)) return "video";
    if (["mp3", "wav", "flac", "aac", "ogg", "m4a", "opus", "wma", "aiff"].includes(e)) return "audio";
    if (["png", "jpg", "jpeg", "webp", "gif", "bmp", "avif", "tiff", "ico", "heic", "tga", "psd"].includes(e)) return "image";
    if (["pdf", "docx", "doc", "md", "html", "txt", "epub", "rtf", "odt"].includes(e)) return "document";
    if (["svg"].includes(e)) return "vector";
    if (["zip", "tar", "gz", "tgz", "directory", "folder"].includes(e)) return "archive";
    if (["csv", "json", "xlsx", "xls", "yaml", "xml", "toml", "sql", "sqlite", "db", "log", "bib", "ics"].includes(e)) return "data";
    return "unknown";
};

export const DropZone: React.FC<DropZoneProps> = ({ files, onAddFiles, onRemoveFile, onClearFiles, onDirectConvert }) => {
    const [scanError, setScanError] = useState<string | null>(null);
    const [isScanning, setIsScanning] = useState(false);
    const scanningRef = useRef(false);
    const addFilesRef = useRef(onAddFiles);
    addFilesRef.current = onAddFiles;
    const [isDragging, setIsDragging] = useState(false);
    const [suggestions, setSuggestions] = useState<FormatOption[]>([]);
    const fileRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
        if (files.length > 0) {
            api.getSmartSuggestions(files[0].extension).then(setSuggestions).catch(() => setSuggestions([]));
        } else {
            setSuggestions([]);
        }
    }, [files]);

    const processFiles = (raw: FileList | null) => {
        if (!raw) return;
        const items = Array.from(raw).slice(0, 256).map(f => ({
            id: crypto.randomUUID(),
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

    const processPaths = async (paths: string[]) => {
        if (scanningRef.current) return;
        scanningRef.current = true;
        setIsScanning(true);
        setScanError(null);
        try {
            const scanned = await api.scanInputs(paths);
            addFilesRef.current(scanned.map(file => {
                const extension = file.is_directory ? "folder" : file.name.split(".").pop()?.toLowerCase() || "";
                return { ...file, id: crypto.randomUUID(), extension, category: detectCategory(extension), status: "pending" as const };
            }));
        } catch (error) { setScanError(String(error)); }
        finally { scanningRef.current = false; setIsScanning(false); }
    };

    useEffect(() => {
        if (!isTauri()) return;

        let disposed = false;
        let unlisten: (() => void) | undefined;
        void import("@tauri-apps/api/window").then(({ getCurrentWindow }) =>
            getCurrentWindow().onDragDropEvent((event) => {
                if (event.payload.type === "over") {
                    setIsDragging(true);
                } else if (event.payload.type === "drop") {
                    setIsDragging(false);
                    void processPaths(event.payload.paths);
                } else {
                    setIsDragging(false);
                }
            }).then((stop) => { if (disposed) stop(); else unlisten = stop; })
        );

        return () => { disposed = true; unlisten?.(); };
    }, []);

    const handleSelectFiles = async (type: "files" | "folder") => {
        if (!isTauri()) {
            fileRef.current?.click();
            return;
        }

        const { open } = await import("@tauri-apps/plugin-dialog");
        const selected = await open({ multiple: true, directory: type === "folder" });
        if (!selected) return;
        await processPaths(Array.isArray(selected) ? selected : [selected]);
    };

    return (
        <div className="space-y-8">
            <div
                onDragOver={(e) => { e.preventDefault(); setIsDragging(true); }}
                onDragLeave={() => setIsDragging(false)}
                onDrop={(e) => {
                    e.preventDefault();
                    setIsDragging(false);
                    if (!isTauri()) processFiles(e.dataTransfer.files);
                }}
                className={`relative h-[240px] border-2 border-dashed rounded-2xl flex flex-col items-center justify-center transition-all ${
                    isDragging
                    ? "border-accent bg-accent/5"
                    : "border-border bg-surface hover:border-border-strong hover:bg-surface-raised"
                }`}
            >
                <input type="file" ref={fileRef} multiple className="hidden" onChange={(e) => processFiles(e.target.files)} />

                <div className="flex flex-col items-center gap-4">
                    <div className={`w-12 h-12 rounded-full flex items-center justify-center bg-surface-raised border border-border ${isDragging ? "text-accent" : "text-text-muted"}`}>
                        <Upload className="w-5 h-5" />
                    </div>
                    <div className="text-center space-y-1">
                        <h3 className="text-[16px] font-semibold text-text-primary">Drop files here</h3>
                        <p className="text-[13px] text-text-muted">or choose files / folders</p>
                    </div>
                    <div className="flex gap-2 mt-2">
                        <button
                            disabled={isScanning}
                            onClick={() => void handleSelectFiles("files").catch(e => setScanError(String(e)))}
                            className="btn-secondary h-[36px]"
                            aria-label="Choose files"
                        >
                            Choose Files
                        </button>
                        <button
                            disabled={isScanning}
                            onClick={() => void handleSelectFiles("folder").catch(e => setScanError(String(e)))}
                            className="btn-secondary h-[36px]"
                            aria-label="Choose folder"
                        >
                            Choose Folder
                        </button>
                    </div>
                </div>
            </div>

            {scanError && <p role="alert" className="text-error">{scanError}</p>}
            {isScanning && <p role="status">Reading selected files…</p>}
            {files.length > 0 && (
                <div className="space-y-6">
                    <div className="flex items-center justify-between">
                        <h3 className="text-[14px] font-semibold text-text-primary">Queue · {files.length} files</h3>
                        <div className="flex gap-2">
                            <button
                                onClick={() => void handleSelectFiles("files")}
                                className="btn-ghost h-[32px] px-2 text-[12px] flex items-center gap-1.5"
                            >
                                <Plus className="w-3.5 h-3.5" />
                                <span>Add files</span>
                            </button>
                            <button
                                onClick={onClearFiles}
                                className="btn-ghost h-[32px] px-2 text-[12px] text-error hover:text-error flex items-center gap-1.5"
                            >
                                <Trash2 className="w-3.5 h-3.5" />
                                <span>Clear</span>
                            </button>
                        </div>
                    </div>

                    <div className="border border-border rounded-lg overflow-hidden divide-y divide-border">
                        <div className="max-h-[300px] overflow-y-auto custom-scrollbar">
                            {files.map(f => (
                                <div key={f.id} className="h-[60px] flex items-center justify-between px-4 bg-surface hover:bg-surface-raised transition-colors group">
                                    <div className="flex items-center gap-4 flex-1 min-w-0">
                                        <FileIcon className="w-5 h-5 text-text-muted shrink-0" />
                                        <div className="truncate flex-1">
                                            <div className="text-[13px] font-medium text-text-primary truncate">{f.name}</div>
                                            <div className="text-[11px] text-text-muted font-mono uppercase">
                                                {(f.size / (1024*1024)).toFixed(2)} MB · {f.extension}
                                            </div>
                                        </div>
                                    </div>
                                    <button
                                        onClick={(e) => { e.stopPropagation(); onRemoveFile(f.id); }}
                                        className="p-2 text-text-muted hover:text-error transition-colors"
                                    >
                                        <X className="w-4 h-4" />
                                    </button>
                                </div>
                            ))}
                        </div>
                    </div>

                    {suggestions.length > 0 && (
                        <div className="space-y-3">
                            <h3 className="text-[14px] font-semibold text-text-primary">Recommended formats</h3>
                            <div className="flex flex-wrap gap-2">
                                {suggestions.map(s => (
                                    <button
                                        key={s.extension}
                                        onClick={() => onDirectConvert?.(s)}
                                        className="btn-secondary h-auto py-2.5 px-4 text-left flex flex-col gap-0.5 border-border hover:border-accent group"
                                    >
                                        <span className="text-[13px] font-semibold text-text-primary group-hover:text-accent">{s.extension.toUpperCase()}</span>
                                        <span className="text-[11px] text-text-muted leading-tight">{s.description || "Compatible format"}</span>
                                    </button>
                                ))}
                            </div>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
};
