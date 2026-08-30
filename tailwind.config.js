/** @type {import('tailwindcss').Config} */
export default {
    content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
    darkMode: "class",
    theme: {
        extend: {
            colors: {
                background: "#0d1117",
                surface: "#161b22",
                "surface-raised": "#21262d",
                border: "#30363d",
                brand: {
                    50: "#eff6ff",
                    100: "#dbeafe",
                    400: "#60a5fa",
                    500: "#3b82f6",
                    600: "#2563eb",
                    700: "#1d4ed8",
                },
            },
        },
    },
    plugins: [],
};
