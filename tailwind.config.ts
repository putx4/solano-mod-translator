import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        bg: {
          DEFAULT: "#0a0a0f",
          elevated: "#12121a",
          card: "rgba(20, 20, 30, 0.6)",
        },
        accent: {
          DEFAULT: "#7c5cff",
          hover: "#9277ff",
          muted: "rgba(124, 92, 255, 0.15)",
        },
        glass: {
          border: "rgba(255, 255, 255, 0.08)",
          highlight: "rgba(255, 255, 255, 0.04)",
        },
        status: {
          success: "#4ade80",
          warning: "#fbbf24",
          error: "#f87171",
          info: "#60a5fa",
        },
      },
      backdropBlur: {
        glass: "24px",
      },
      boxShadow: {
        glass: "0 8px 32px rgba(0, 0, 0, 0.4), inset 0 1px 0 rgba(255, 255, 255, 0.05)",
        glow: "0 0 40px rgba(124, 92, 255, 0.3)",
      },
      animation: {
        "fade-in": "fadeIn 0.3s ease-out",
        "slide-up": "slideUp 0.4s cubic-bezier(0.16, 1, 0.3, 1)",
        "pulse-glow": "pulseGlow 2s ease-in-out infinite",
        "float": "float 6s ease-in-out infinite",
      },
      keyframes: {
        fadeIn: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        slideUp: {
          "0%": { opacity: "0", transform: "translateY(20px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        pulseGlow: {
          "0%, 100%": { boxShadow: "0 0 20px rgba(124, 92, 255, 0.2)" },
          "50%": { boxShadow: "0 0 40px rgba(124, 92, 255, 0.5)" },
        },
        float: {
          "0%, 100%": { transform: "translateY(0)" },
          "50%": { transform: "translateY(-10px)" },
        },
      },
    },
  },
  plugins: [],
} satisfies Config;