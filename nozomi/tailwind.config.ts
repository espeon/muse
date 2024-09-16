import type { Config } from "tailwindcss";

const config = {
  darkMode: "media",
  content: [
    "./pages/**/*.{ts,tsx}",
    "./components/**/*.{ts,tsx}",
    "./app/**/*.{ts,tsx}",
    "./src/**/*.{ts,tsx}",
  ],
  prefix: "",
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      gridTemplateAreas: {
        "main-desktop": [
          "left-sidebar    main-view         right-sidebar",
          "now-playing-bar now-playing-bar now-playing-bar",
        ],
        "main-mobile": ["main-view", "now-playing-bar", "nav"],
      },
      fontFamily: {
        sans: ["var(--font-fig)"],
        mono: ["var(--font-mono)"],
      },
      colors: {
        aoi: {
          "50": "oklch(0.963 0.011 312.814)",
          "100": "oklch(0.925 0.021 312.814)",
          "200": "oklch(0.851 0.042 312.814)",
          "300": "oklch(0.776 0.104 312.814)",
          "400": "oklch(0.701 0.135 312.814)",
          "500": "oklch(0.627 0.085 312.814)",
          "600": "oklch(0.507 0.071 312.814)",
          "700": "oklch(0.380 0.069 312.814)",
          "800": "oklch(0.253 0.067 312.814)",
          "900": "oklch(0.127 0.064 312.814)",
          "950": "oklch(0.063 0.063 312.814)",
        },
        wisteria: {
          "50": "oklch(0.955 0.012 314.384)",
          "100": "oklch(0.910 0.025 314.384)",
          "200": "oklch(0.820 0.050 314.384)",
          "300": "oklch(0.730 0.105 314.384)",
          "400": "oklch(0.640 0.120 314.384)",
          "500": "oklch(0.550 0.100 314.384)",
          "600": "oklch(0.445 0.084 314.384)",
          "700": "oklch(0.334 0.081 314.384)",
          "800": "oklch(0.223 0.078 314.384)",
          "900": "oklch(0.111 0.076 314.384)",
          "950": "oklch(0.056 0.074 314.384)",
        },
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
      },
      keyframes: {
        "accordion-down": {
          from: { height: "0" },
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: "0" },
        },
        flip: {
          to: {
            transform: "rotate(360deg)",
          },
        },
        kitrotate: {
          from: {
            transform: "rotate(0deg)",
          },
          to: {
            transform: "rotate(360deg)",
          },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
        flip: "flip 12s infinite steps(2, end)",
        spinslow: "spin 12s ease-in-out infinite",
        kitrotate: "kitrotate 6s linear infinite both",
      },
    },
  },
  plugins: [
    require("tailwindcss-animate"),
    require("@savvywombat/tailwindcss-grid-areas"),
  ],
} satisfies Config;

export default config;
