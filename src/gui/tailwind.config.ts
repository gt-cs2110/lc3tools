import { Config } from "tailwindcss";
import primeui from "tailwindcss-primeui";

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
  plugins: [primeui],
} satisfies Config;