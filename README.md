![Logo](res/img/logo.png)


# Rust-eze Robotics' AI: Artemisia

![Tags](https://badgen.net/badge/icon/%23AdvancedProgramming%20%23AI%20%23ArtemisIA/14406F1?icon=https://icons.getbootstrap.com/assets/icons/bookmarks-fill.svg&label&labelColor=FFF) ![Language](https://img.shields.io/badge/Built_with-Rust-F86424?labelColor=000&logo=rust) ![Version](https://badgen.net/badge/Version/01.01/F08C2F?labelColor=000) ![GroupName](https://badgen.net/badge/Group%20Name/Rust-eze%20Robotics/A62424?labelColor=000) ![Author](https://badgen.net/badge/Author/Chiara%20S./F23A29?labelColor=000)

**"Revolutionary AI meets timeless art, in the most unexpected way. Bringing passion to the world, one tree at a time."**

---

# Summary

- [Rust-eze Robotics' AI: ArtemisIA](#rusteze-robotics-ai-artemisia)
- [Summary](#summary)
- [Description](#description)
- [Requirements](#requirements)
- [Notes](#notes)

---

# Description

This AI is inspired by some great artists and takes her name from Artemisia Gentileschi. Her main goal is to paint on the world using trees and rocks, explore the environment and collect materials to create fine art, thanks to the `spyglass`, `sense_and_find_by_rustafariani` and `OhCrab_collection` tools.
She will use them to paint the world, through the `giotto_tool`.

The AI is implemented in Rust and uses the `robotics_lib` library to interact with the environment.

The AI is implemented as a state machine, with the following states:
- `INIT`: the AI initializes itself, setting all the necessary parameters
- `CHILL`: the AI wanders around for a while, exploring the world with the `spyglass` tool, to get inspired for her next masterpiece
- `DETECT`: the AI detects materials, using the `sense_and_find_by_rustafariani` tool
- `GATHER`: the AI gathers materials, using the `OhCrab_collection` tool
- `PAINT`: the AI paints the world, using the `giotto_tool`
- `DIE`: the AI terminates her existence, with a final painting

---

# Requirements

- ![Rust](https://img.shields.io/badge/Rust-F86424?labelColor=000&logo=rust) <sup>([Install](https://www.rust-lang.org/tools/install))
- [robotics_lib](https://advancedprogramming.disi.unitn.it/crate?name=robotics_lib)
- [giotto_tool](https://advancedprogramming.disi.unitn.it/crate?name=giotto_tool)
- [spyglass](https://advancedprogramming.disi.unitn.it/crate?name=spyglass)
- [sense_and_find_by_rustafariani](https://advancedprogramming.disi.unitn.it/crate?name=sense_and_find_by_rustafariani)
- [OhCrab_collection](https://advancedprogramming.disi.unitn.it/crate?name=OhCrab_collection)
- [bob_lib](https://advancedprogramming.disi.unitn.it/crate?name=bob_lib)

---

# Notes

- Artemis-IA doesn't work properly, because there are some *unresolvable* issues in the painting phase. What a shame.

---

# Concept Poster

![Poster](res/img/concept_poster.png)

---

Made with ♡ by
[![Chiara S.](https://badgen.net/badge/icon/Chiara%20S./B67DFF?icon=github&label&labelColor=000)](https://github.com/chiarasabaini)
