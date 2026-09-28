import React from "react";
import { AlertTriangle, RotateCcw } from "lucide-react";

interface ErrorBoundaryState {
    error: Error | null;
}

export class ErrorBoundary extends React.Component<React.PropsWithChildren, ErrorBoundaryState> {
    state: ErrorBoundaryState = { error: null };

    static getDerivedStateFromError(error: Error): ErrorBoundaryState {
        return { error };
    }

    componentDidCatch(error: Error, info: React.ErrorInfo) {
        console.error("File2File UI crashed", error, info);
    }

    render() {
        if (!this.state.error) return this.props.children;
        return (
            <main className="min-h-screen bg-background text-text-primary flex items-center justify-center p-8">
                <div role="alert" className="max-w-lg border border-error/40 bg-surface rounded-lg p-6 space-y-4">
                    <AlertTriangle className="w-6 h-6 text-error" />
                    <div className="space-y-1">
                        <h1 className="text-[18px] font-semibold">File2File could not display this screen</h1>
                        <p className="text-[13px] text-text-secondary">
                            Your files were not uploaded. Reload the local interface to recover; active desktop jobs will still be cancelled safely when the app exits.
                        </p>
                    </div>
                    <details className="text-[11px] text-text-muted">
                        <summary className="cursor-pointer">Technical details</summary>
                        <pre className="mt-2 whitespace-pre-wrap select-text">{this.state.error.message}</pre>
                    </details>
                    <button onClick={() => window.location.reload()} className="btn-primary flex items-center gap-2">
                        <RotateCcw className="w-4 h-4" />
                        Reload File2File
                    </button>
                </div>
            </main>
        );
    }
}
