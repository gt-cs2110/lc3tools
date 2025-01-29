import { Config } from "tailwindcss";

export default {
  darkMode: "selector",
  content: ["./src/**/*.{html,js,vue,ts}"],
  theme: {
    extend: {
      fontFamily: {
        "mono": ["Consolas", "Menlo", "Courier", "monospace"]
      }
    },
  },
  plugins: [],
} satisfies Config;