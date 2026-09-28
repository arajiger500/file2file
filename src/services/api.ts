import {
    HardwareInfo,
    FormatOption,
    QuickPreset,
    SidecarHealthReport,
    ConversionRequest,
    ConversionResult,
    ValidationResult,
    BatchValidationItem,
} from "../types";
import { handleUniversalEngine } from "./universal-engine";

export const isTauri = () =>
    typeof window !== "undefined" &&
    Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);

async function invokeTauri<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        return invoke<T>(cmd, args);
    }
    return handleUniversalEngine<T>(cmd, args);
}

export const api = {
    scanInputs: (paths: string[]) => invokeTauri<{ path: string; name: string; size: number; is_directory: boolean }[]>("scan_inputs", { paths }),
    detectHardware: () => invokeTauri<HardwareInfo>("detect_hardware"),
    getCompatibleTargets: (inputExt: string) =>
        invokeTauri<FormatOption[]>("get_compatible_targets", { inputExt }),
    getSmartSuggestions: (inputExt: string) =>
        invokeTauri<FormatOption[]>("get_smart_suggestions", { inputExt }),
    getPresets: () => invokeTauri<QuickPreset[]>("get_presets"),
    checkSidecars: () => invokeTauri<SidecarHealthReport>("check_sidecars"),
    validateJob: (inputPath: string, targetFormat: string) =>
        invokeTauri<ValidationResult>("validate_job", { inputPath, targetFormat }),
    validateJobs: (jobId: string, inputPaths: string[], targetFormat: string) =>
        invokeTauri<BatchValidationItem[]>("validate_jobs", { jobId, inputPaths, targetFormat }),
    startConversion: (request: ConversionRequest) =>
        invokeTauri<ConversionResult>("start_conversion", { request }),
    cancelJob: (jobId: string) =>
        invokeTauri<void>("cancel_job", { jobId }),
    showInFolder: (path: string) => invokeTauri<void>("show_in_folder", { path }),
    chooseOutputDirectory: async (): Promise<string | null> => {
        if (!isTauri()) return null;
        const { open } = await import("@tauri-apps/plugin-dialog");
        const selected = await open({ directory: true, multiple: false, title: "Choose output folder" });
        return typeof selected === "string" ? selected : null;
    },
    downloadFile: async (result: ConversionResult, fallbackFilename?: string) => {
        const filename = fallbackFilename || result.output_path.split("/").pop() || "file2file_output";
        if (result.download_url) {
            const a = document.createElement("a");
            a.href = result.download_url;
            a.download = filename;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
        }
    },
};

export const downloadFile = api.downloadFile;
