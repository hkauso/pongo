/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./templates/**/*.html"],
    darkMode: "class",
    theme: {
        extend: {
            colors: {
                brand: "rgb(16 185 129)",
                "brand-low": "rgb(16 185 129)",
            },
            animation: {
                "fade-in": "fadein 0.25s ease-in-out 1 running",
            },
            keyframes: {
                fadein: {
                    "0%": { opacity: "0%" },
                    "100%": { opacity: "100%" },
                },
            },
        },
    },
    plugins: [],
};
