import JSZip from "jszip";
import {
    FormatOption,
    ConversionRequest,
} from "../types";

const cancelledJobs = new Set<string>();

/**
 * BROWSER FALLBACK ENGINE
 * Provides limited, truthful local transformations in the browser when Tauri is unavailable.
 */
export async function handleUniversalEngine<T>(cmd: string, args?: Record<string, any>): Promise<T> {
    if (cmd === "detect_hardware") {
        return {
            gpu_vendor: "No GPU access (Browser Fallback)",
            hardware_acceleration_supported: false,
            recommended_encoder: "wasm",
            cpu_cores: navigator.hardwareConcurrency || 1,
            available_encoders: [],
            detected_gpus: []
        } as unknown as T;
    }

    if (cmd === "get_compatible_targets" || cmd === "get_smart_suggestions") {
        const ext = ((args?.inputExt as string) || "png").toLowerCase();
        const formats = getFullFormatCatalog(ext);
        return (cmd === "get_smart_suggestions" ? formats.slice(0, 4) : formats) as unknown as T;
    }

    if (cmd === "validate_job") {
        const input = String(args?.inputPath || "");
        const extension = input.split(".").pop()?.toLowerCase() || "";
        const supported = getFullFormatCatalog(extension).some(format => format.extension === String(args?.targetFormat || "").toLowerCase());
        return { is_valid: supported, warnings: [], error: supported ? null : "This browser conversion is not supported.", file_info: null } as unknown as T;
    }

    if (cmd === "validate_jobs") {
        const jobId = String(args?.jobId || "");
        if (cancelledJobs.delete(jobId)) throw new Error("Validation cancelled");
        const inputPaths = Array.isArray(args?.inputPaths) ? args.inputPaths.map(String).slice(0, 256) : [];
        const targetFormat = String(args?.targetFormat || "").toLowerCase();
        return inputPaths.map(inputPath => {
            const extension = inputPath.split(".").pop()?.toLowerCase() || "";
            const supported = getFullFormatCatalog(extension).some(format => format.extension === targetFormat);
            return {
                input_path: inputPath,
                result: {
                    is_valid: supported,
                    warnings: [],
                    error: supported ? null : "This browser conversion is not supported.",
                    file_info: null,
                },
            };
        }) as unknown as T;
    }

    if (cmd === "cancel_job") {
        cancelledJobs.add(String(args?.jobId || ""));
        return undefined as T;
    }

    if (cmd === "get_presets") {
        return [
            { id: "pre-1", title: "PDF to Editable DOCX", target_name: "DOCX", from_category: "document", to_format: "docx", description: "Text extraction with limited layout fidelity", badge: "Web", icon: "file-text" },
            { id: "pre-2", title: "Image to WebP", target_name: "WebP", from_category: "image", to_format: "webp", description: "Canvas (Browser)", badge: "Web", icon: "image" }
        ] as unknown as T;
    }

    if (cmd === "start_conversion") {
        const req = args?.request as ConversionRequest;
        const targetExt = req.target_format.toLowerCase();
        if (!req.rawFile) return { success: false, error: "Empty File" } as unknown as T;
        if (req.rawFile.size > 64 * 1024 * 1024) return { success: false, error: "Browser conversion is limited to 64 MiB inputs." } as unknown as T;
        const jobId = req.job_id || crypto.randomUUID();
        cancelledJobs.delete(jobId);

        const startTime = Date.now();
        let finalBlob: Blob;

        try {
            const sourceExt = req.rawFile.name.split(".").pop()?.toLowerCase() || "";
            const textSource = ["txt", "md", "html"].includes(sourceExt);

            if (targetExt === "docx" && sourceExt === "pdf") {
                finalBlob = await runRealPdfToDocx(req.rawFile, () => cancelledJobs.has(jobId));
            } else if (targetExt === "json" && sourceExt === "csv") {
                finalBlob = new Blob([await runCsvToJson(req.rawFile)], { type: "application/json" });
            } else if (targetExt === "csv" && sourceExt === "json") {
                finalBlob = new Blob([await runJsonToCsv(req.rawFile)], { type: "text/csv" });
            } else if (["webp", "jpg", "png"].includes(targetExt) && req.rawFile.type.startsWith("image/")) {
                finalBlob = await runRealImageTranscode(req.rawFile, targetExt);
            } else if (targetExt === "pdf") {
                if (req.rawFile.type.startsWith("image/")) {
                    finalBlob = await runImageToPdf(req.rawFile);
                } else if (textSource) {
                    finalBlob = await runTextToPdf(req.rawFile);
                } else {
                    throw new Error("This browser conversion requires a plain-text or image source.");
                }
            } else if (["md", "txt", "html"].includes(targetExt) && textSource) {
                finalBlob = new Blob([await runTextExtraction(req.rawFile, targetExt)], { type: targetExt === "html" ? "text/html" : "text/plain" });
            } else {
                throw new Error("This conversion requires the File2File desktop app and its local conversion tools.");
            }
        } catch (e) {
            cancelledJobs.delete(jobId);
            return { success: false, error: String(e) } as unknown as T;
        }

        if (cancelledJobs.delete(jobId)) {
            return { job_id: jobId, input_path: req.input_path, output_path: "", success: false, original_size_bytes: req.rawFile.size, converted_size_bytes: 0, elapsed_ms: Date.now() - startTime, error: "Conversion cancelled" } as unknown as T;
        }

        return {
            job_id: jobId,
            input_path: req.input_path,
            output_path: `file2file_output.${targetExt}`,
            success: true,
            original_size_bytes: req.rawFile.size,
            converted_size_bytes: finalBlob.size,
            elapsed_ms: Date.now() - startTime,
            download_url: URL.createObjectURL(finalBlob),
        } as unknown as T;
    }

    if (cmd === "check_sidecars") {
        const missing = (name: string) => ({ name, available: false, version: null, path_or_sidecar: "browser", error: "Desktop engine unavailable in browser mode" });
        return {
            binaries: [],
            categories: [
                { category: "Video", ready: false, engine: "N/A", message: "Desktop required" },
                { category: "Audio", ready: false, engine: "N/A", message: "Desktop required" },
                { category: "Images", ready: true, engine: "Canvas", message: "Basic support" },
                { category: "Documents", ready: true, engine: "jsPDF/PDF.js", message: "Limited fidelity" },
                { category: "Data", ready: true, engine: "JS-Logic", message: "CSV and JSON only" }
            ],
            all_ready: false,
            ffmpeg: missing("ffmpeg"),
            ffprobe: missing("ffprobe"),
            pandoc: missing("pandoc"),
            magick: missing("magick"),
            imagemagick: missing("magick"),
            pdftotext: missing("pdftotext"),
            pdftohtml: missing("pdftohtml"),
        } as unknown as T;
    }

    return [] as unknown as T;
}

async function runRealPdfToDocx(file: File, cancelled: () => boolean): Promise<Blob> {
    const pdfjsLib = await import("pdfjs-dist");
    pdfjsLib.GlobalWorkerOptions.workerSrc = new URL("../../node_modules/pdfjs-dist/build/pdf.worker.min.mjs", import.meta.url).href;
    const pdf = await pdfjsLib.getDocument({ data: await file.arrayBuffer() }).promise;
    let text = "";
    for (let i = 1; i <= pdf.numPages; i++) {
        if (cancelled()) throw new Error("Conversion cancelled");
        const page = await pdf.getPage(i);
        const content = await page.getTextContent();
        // @ts-ignore
        text += content.items.map((it: any) => it.str).join(" ") + "\n";
    }
    const zip = new JSZip();
    const cleanText = text.replace(/[<>&]/g, c => ({'<':'&lt;','>':'&gt;','&':'&amp;'}[c] || c));
    zip.file("word/document.xml", `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>${cleanText.split("\n").map(l => `<w:p><w:r><w:t>${l}</w:t></w:r></w:p>`).join("")}</w:body></w:document>`);
    zip.file("_rels/.rels", `<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="r1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>`);
    zip.file("[Content_Types].xml", `<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>`);
    return await zip.generateAsync({ type: "blob" });
}

async function runRealImageTranscode(file: File, ext: string): Promise<Blob> {
    return new Promise((res, rej) => {
        const img = new Image();
        const source = URL.createObjectURL(file);
        img.onload = () => {
            if (img.width * img.height > 100_000_000) {
                URL.revokeObjectURL(source);
                rej("Image exceeds the 100 megapixel browser limit");
                return;
            }
            const canvas = document.createElement("canvas");
            canvas.width = img.width; canvas.height = img.height;
            const ctx = canvas.getContext("2d");
            if (ext === "jpg" && ctx) {
                ctx.fillStyle = "#ffffff";
                ctx.fillRect(0, 0, canvas.width, canvas.height);
            }
            ctx?.drawImage(img, 0, 0);
            const mime = ext === "jpg" ? "image/jpeg" : `image/${ext}`;
            canvas.toBlob(b => {
                URL.revokeObjectURL(source);
                b ? res(b) : rej("Transcode failed");
            }, mime, 0.9);
        };
        img.onerror = () => { URL.revokeObjectURL(source); rej("Image decode failed"); };
        img.src = source;
    });
}

async function runImageToPdf(file: File): Promise<Blob> {
    const dimensions = await new Promise<{ width: number; height: number }>((resolve, reject) => {
        const image = new Image();
        const source = URL.createObjectURL(file);
        image.onload = () => {
            URL.revokeObjectURL(source);
            resolve({ width: image.width, height: image.height });
        };
        image.onerror = () => {
            URL.revokeObjectURL(source);
            reject(new Error("Image decode failed"));
        };
        image.src = source;
    });
    if (dimensions.width * dimensions.height > 100_000_000) {
        throw new Error("Image exceeds the 100 megapixel browser limit");
    }
    const { jsPDF } = await import("jspdf");
    const doc = new jsPDF();
    const jpeg = await runRealImageTranscode(file, "jpg");
    const data = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = event => resolve(event.target?.result as string);
        reader.onerror = () => reject(new Error("Image could not be read"));
        reader.readAsDataURL(jpeg);
    });
    doc.addImage(data, "JPEG", 10, 10, 190, 0);
    return doc.output("blob");
}

async function runTextToPdf(file: File): Promise<Blob> {
    const { jsPDF } = await import("jspdf");
    const doc = new jsPDF();
    const text = await file.text();
    const pageWidth = doc.internal.pageSize.getWidth();
    const pageHeight = doc.internal.pageSize.getHeight();
    const lines = doc.splitTextToSize(text, pageWidth - 20) as string[];
    let y = 14;
    for (const line of lines) {
        if (y > pageHeight - 12) {
            doc.addPage();
            y = 14;
        }
        doc.text(line, 10, y);
        y += 6;
    }
    return doc.output("blob");
}

async function runTextExtraction(file: File, target: string): Promise<string> {
    const raw = await file.text();
    const escapeHtml = (value: string) => value.replace(/[&<>"']/g, char => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "\"": "&quot;", "'": "&#39;" }[char] || char));
    if (target === "html") return `<html><body><h3>${escapeHtml(file.name)}</h3><hr><p>${escapeHtml(raw).replace(/\n/g, "<br>")}</p></body></html>`;
    return raw;
}

async function runCsvToJson(file: File): Promise<string> {
    const text = await file.text();
    const rows: string[][] = [];
    let row: string[] = [];
    let field = "";
    let inQuotes = false;
    let afterQuote = false;
    for (let i = 0; i < text.length; i++) {
        const char = text[i];
        if (char === '"') {
            if (inQuotes && text[i + 1] === '"') {
                field += '"';
                i++;
            } else if (inQuotes) {
                inQuotes = false;
                afterQuote = true;
            } else if (field.length === 0 && !afterQuote) {
                inQuotes = true;
            } else {
                throw new Error("Malformed CSV: quote inside an unquoted field");
            }
        } else if (char === "," && !inQuotes) {
            row.push(field);
            field = "";
            afterQuote = false;
        } else if ((char === "\n" || char === "\r") && !inQuotes) {
            if (char === "\r" && text[i + 1] === "\n") i++;
            row.push(field);
            if (row.some(cell => cell.length > 0)) rows.push(row);
            row = [];
            field = "";
            afterQuote = false;
        } else {
            if (afterQuote) throw new Error("Malformed CSV: unexpected text after a quoted field");
            field += char;
        }
    }
    if (inQuotes) throw new Error("Malformed CSV: unterminated quoted field");
    row.push(field);
    if (row.some(cell => cell.length > 0)) rows.push(row);
    if (rows.length === 0) return "[]";

    const headers = rows[0];
    if (headers.some(header => !header) || new Set(headers).size !== headers.length) {
        throw new Error("CSV headers must be non-empty and unique");
    }
    const data = rows.slice(1).map(values => {
        if (values.length !== headers.length) throw new Error("Malformed CSV: row width does not match headers");
        const obj: any = {};
        headers.forEach((header, i) => {
            if (header) obj[header] = values[i] || "";
        });
        return obj;
    });
    return JSON.stringify(data, null, 2);
}

async function runJsonToCsv(file: File): Promise<string> {
    const text = await file.text();
    const data = JSON.parse(text);
    if (!Array.isArray(data) || data.length === 0 || data.some(item => typeof item !== "object" || item === null || Array.isArray(item))) {
        throw new Error("JSON-to-CSV requires a non-empty array of objects");
    }

    // Collect all unique keys for headers
    const headersSet = new Set<string>();
    data.forEach(item => {
        if (typeof item === 'object' && item !== null) {
            Object.keys(item).forEach(k => headersSet.add(k));
        }
    });

    const headers = Array.from(headersSet).sort();
    if (headers.length === 0) return "";

    const csvCell = (value: unknown): string => {
        if (typeof value === "object" && value !== null) {
            throw new Error("Nested JSON values cannot be represented safely in CSV");
        }
        let text = String(value ?? "");
        if (/^[=+\-@\t\r]/.test(text)) text = `'${text}`;
        return /[,\n\r"]/.test(text) ? `"${text.replace(/"/g, '""')}"` : text;
    };
    const csvLines = [
        headers.map(csvCell).join(","),
        ...data.map(item => {
            return headers.map(h => {
                const val = item[h];
                if (val === undefined || val === null) return "";
                return csvCell(val);
            }).join(",");
        })
    ];
    return csvLines.join("\n");
}

function getFullFormatCatalog(ext: string): FormatOption[] {
    const isDoc = ["pdf", "docx", "doc", "md", "txt", "html", "epub", "rtf", "odt"].includes(ext);
    const isImg = ["png", "jpg", "jpeg", "webp", "avif", "bmp", "tiff", "ico", "heic", "tga", "psd"].includes(ext);

    const catalog: FormatOption[] = [];

    if (isDoc) {
        let docTargets: string[] = [];
        if (ext === "pdf") {
            docTargets = ["docx", "txt", "html"];
        } else if (["md", "txt", "html"].includes(ext)) {
            docTargets = ["pdf", "md", "html", "txt"];
        }

        docTargets.forEach(t => {
            if (t === ext) return;
            catalog.push({
                extension: t,
                name: `${t.toUpperCase()} Document`,
                category: "document",
                subcategory: "Universal",
                description: `Browser-based ${t.toUpperCase()} export`,
                comparison_note: ext === "pdf" ? "Basic text extraction only" : null,
                is_lossless: false,
                is_recommended: true,
                recommended_for: ["Editing", "Sharing"],
                sidecar_engine: "Browser-Core",
                pros: ["No install needed"],
                cons: ["Limited fidelity"]
            });
        });
    }

    if (isImg) {
        const imgTargets = ["webp", "png", "jpg", "pdf"];
        imgTargets.forEach(t => {
            if (t === ext) return;
            catalog.push({
                extension: t,
                name: `${t.toUpperCase()} Image`,
                category: t === "pdf" ? "document" : "image",
                subcategory: "Graphics",
                description: `Canvas-based ${t.toUpperCase()} encoding`,
                comparison_note: null,
                is_lossless: false,
                is_recommended: true,
                recommended_for: ["Web"],
                sidecar_engine: "Browser-Canvas",
                pros: ["Fast"],
                cons: []
            });
        });
    }

    if (ext === "csv" || ext === "json") {
        const targets = ext === "csv" ? ["json"] : ["csv"];
        targets.forEach(t => {
            catalog.push({
                extension: t,
                name: `${t.toUpperCase()} Data`,
                category: "data",
                subcategory: "Universal",
                description: `JS-based ${t.toUpperCase()} conversion`,
                comparison_note: null,
                is_lossless: false,
                is_recommended: true,
                recommended_for: ["Analysis"],
                sidecar_engine: "JS-Logic",
                pros: ["Runs locally"],
                cons: ["CSV cannot preserve JSON types"]
            });
        });
    }

    return catalog;
}
