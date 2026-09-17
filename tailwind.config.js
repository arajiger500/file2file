/** @type {import('tailwindcss').Config} */
export default {
    content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
    darkMode: "class",
    theme: {
        extend: {
            colors: {
                background: "#111111",
                surface: "#181818",
                "surface-raised": "#202020",
                "surface-elevated": "#252525",
                border: "#303030",
                "border-strong": "#3A3A3A",
                "text-primary": "#F2F2F2",
                "text-secondary": "#B0B0B0",
                "text-muted": "#777777",
                "text-disabled": "#555555",
                accent: "#4C8DFF",
                success: "#4CAF6A",
                warning: "#D99A32",
                error: "#D65C5C",
            },
            borderRadius: {
                DEFAULT: "8px",
                md: "10px",
                lg: "12px",
                xl: "14px",
                "2xl": "16px",
            },
            fontFamily: {
                sans: ['Inter', 'system-ui', '-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'sans-serif'],
                mono: ['ui-monospace', 'SFMono-Regular', 'Menlo', 'Monaco', 'Consolas', '"Liberation Mono"', '"Courier New"', 'monospace'],
            },
        },
    },
    plugins: [],
};
