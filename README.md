
# Juzo | Chat Manager

Juzo is a chat manager developed in Rust.
- [Juzo | Chat Manager](https://t.me/juzo_cm_bot)


## Project Structure & Licensing

Juzo is split into multiple crates with different access levels:

### `crates/juzo-core`
- Contains the internal logic
- **Proprietary, closed-source component**
- Not intended for direct public use or distribution

### `crates/juzo`
- Provides the **main interaction with Telegram**
- **Source-available, but not open-source**
- Distributed under a **proprietary license**
- Full license terms are available in the `LICENSE` file
---
© 2025–2026 JuzoCode. All rights reserved.