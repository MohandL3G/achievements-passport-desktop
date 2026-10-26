# 🎮 Achievements Passport Generator

A desktop application written in **Rust** that generates futuristic, sci-fi HUD achievement passport cards from the [Achievements](https://github.com/PSerban93/Achievements) application.

Inspired by the Palworld Passport layout, it aggregates your local game stats into a shareable passport card featuring an overall rating badge (**OVR**), progress telemetry, and library milestones.

---

## Highlights

- **Automatic Ingestion**: Reads your local library stats directly from `%APPDATA%\Achievements\dashboard-summary.json`.
- **Two Passport Modes**:
  - **Overall Library**: An all-in-one passport summarizing total completions, 100% perfect games, in-progress titles, and overall completion rate.
  - **Single Game**: Select any title (e.g. *Palworld*, *Black Myth: Wukong*, *The Witcher 3*) to generate a dedicated game card.
- **HUD Themes**: Includes Cyberpunk Cyan (Palworld style), Trophy Gold, Bloodborne Crimson, Emerald Matrix, and Neon Violet.
- **Live Preview & Export**: Preview cards in real time within the UI and export to high-resolution PNG (1200 × 850) or vector SVG.

---

## Automated Builds

Pre-built binaries for **Windows** (`.exe`) and **Linux** are automatically built on every commit via GitHub Actions. You can download the latest package from the repository's **Actions** tab.
