import {
    HardwareInfo,
    FormatOption,
    QuickPreset,
    SidecarHealthReport,
    ConversionRequest,
    ConversionResult,
    ValidationResult,
} from "../types";
// ... (rest of imports)

// ...

export const api = {
    detectHardware: () => invokeTauri<HardwareInfo>("detect_hardware"),
    getCompatibleTargets: (inputExt: string) =>
        invokeTauri<FormatOption[]>("get_compatible_targets", { inputExt }),
    getSmartSuggestions: (inputExt: string) =>
        invokeTauri<FormatOption[]>("get_smart_suggestions", { inputExt }),
    getPresets: () => invokeTauri<QuickPreset[]>("get_presets"),
    checkSidecars: () => invokeTauri<SidecarHealthReport>("check_sidecars"),
    validateJob: (inputPath: string, targetFormat: string) =>
        invokeTauri<ValidationResult>("validate_job", { inputPath, targetFormat }),
    startConversion: (request: ConversionRequest) =>
// ...
        invokeTauri<ConversionResult>("start_conversion", { request }),
    downloadFile: async (result: ConversionResult, fallbackFilename?: string) => {
        const filename = fallbackFilename || result.output_path.split("/").pop() || "file2file_output";
        if (isTauri() && !result.download_url) {
            try {
                // @ts-ignore
                const { save } = await import("@tauri-apps/plugin-dialog");
                const savePath = await save({ defaultPath: filename });
                if (!savePath) return;
                await invokeTauri<void>("copy_file", { src: result.output_path, dest: savePath });
                // @ts-ignore
                const { open } = await import("@tauri-apps/plugin-shell");
                await open(savePath);
            } catch (e) { console.error("Save error:", e); }
            return;
        }
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
