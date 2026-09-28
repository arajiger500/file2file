import {
    AdvancedSettings,
    BinaryStatus,
    FormatOption,
    SidecarHealthReport,
} from "../types";

export const DEFAULT_SETTINGS: AdvancedSettings = {
    crf: 23,
    resolution: "original",
    hardwareAccel: false,
    selectedEncoder: "auto",
    stripMetadata: true,
    audioBitrate: "192k",
    collisionPolicy: "autorename",
    maxParallelJobs: 4,
};

export function sanitizeSettings(value: unknown): AdvancedSettings {
    if (!value || typeof value !== "object" || Array.isArray(value)) return { ...DEFAULT_SETTINGS };
    const candidate = value as Partial<AdvancedSettings>;
    const collisionPolicies = ["overwrite", "autorename", "skip"];
    const resolution = typeof candidate.resolution === "string" &&
        (candidate.resolution === "original" || /^\d{2,4}x\d{2,4}$/.test(candidate.resolution))
        ? candidate.resolution
        : DEFAULT_SETTINGS.resolution;
    const audioBitrates = ["320k", "256k", "192k", "128k", "96k", "64k"];
    const encoders = [
        "auto", "libx264", "libx265", "h264_nvenc", "hevc_nvenc",
        "h264_qsv", "h264_amf", "h264_videotoolbox",
    ];

    return {
        crf: Math.max(0, Math.min(51, Math.round(Number(candidate.crf) || DEFAULT_SETTINGS.crf))),
        resolution,
        hardwareAccel: candidate.hardwareAccel === true,
        selectedEncoder: encoders.includes(String(candidate.selectedEncoder))
            ? String(candidate.selectedEncoder)
            : "auto",
        stripMetadata: candidate.stripMetadata !== false,
        audioBitrate: audioBitrates.includes(String(candidate.audioBitrate))
            ? String(candidate.audioBitrate)
            : DEFAULT_SETTINGS.audioBitrate,
        collisionPolicy: collisionPolicies.includes(String(candidate.collisionPolicy))
            ? candidate.collisionPolicy as AdvancedSettings["collisionPolicy"]
            : DEFAULT_SETTINGS.collisionPolicy,
        maxParallelJobs: Math.max(1, Math.min(4, Math.floor(Number(candidate.maxParallelJobs) || 4))),
    };
}

export function intersectFormatGroups(groups: FormatOption[][]): FormatOption[] {
    if (groups.length === 0) return [];
    const common = groups.slice(1).reduce(
        (extensions, group) => new Set(
            [...extensions].filter(extension => group.some(format => format.extension === extension)),
        ),
        new Set(groups[0].map(format => format.extension)),
    );
    return groups[0].filter(format => common.has(format.extension));
}

export function getEngineIssue(
    format: FormatOption,
    sidecars: SidecarHealthReport | null,
): string | null {
    const engine = format.sidecar_engine.toLowerCase();
    const externalNames = ["ffmpeg", "imagemagick", "pandoc", "poppler"];
    if (!externalNames.some(name => engine.includes(name))) return null;
    if (!sidecars) return "Engine status is unavailable; refresh diagnostics";

    const requirements: Array<[boolean, BinaryStatus | undefined, string]> = [
        [engine.includes("ffmpeg"), sidecars.ffmpeg, "FFmpeg"],
        [engine.includes("ffmpeg"), sidecars.ffprobe, "FFprobe"],
        [engine.includes("imagemagick"), sidecars.imagemagick || sidecars.magick, "ImageMagick"],
        [engine.includes("pandoc"), sidecars.pandoc, "Pandoc"],
        [engine.includes("poppler") && format.extension !== "txt", sidecars.pdftohtml, "pdftohtml"],
        [engine.includes("poppler") && format.extension === "txt", sidecars.pdftotext, "pdftotext"],
    ];
    const problem = requirements.find(([needed, status]) => needed && !status?.available);
    if (!problem) return null;
    const [, status, name] = problem;
    return status?.error
        ? `${name} is unusable: ${status.error}`
        : `${name} is not installed or was not found`;
}
