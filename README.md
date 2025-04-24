# 🪄 Rune Etching Rust

Etch your own [Bitcoin Runes](https://docs.ordinals.com/runes.html) tokens directly on-chain using this Rust-based CLI tool.

> Powered by Rust + BDK + OP_RETURN magic  
> Designed for Bitcoin Testnet rune experimentation 🧪

---

## 🚀 Features

- ⚙️ Etch Runes on Bitcoin using `OP_RETURN`
- 🔐 Uses Bitcoin Descriptors via [`bdk`](https://github.com/bitcoindevkit/bdk)
- 🧪 Supports **Testnet**
- ✍️ Simple CLI tool to build, sign, and broadcast etching transactions

---

## 📦 Installation

### ✅ Prerequisites
- Rust & Cargo: [https://rustup.rs](https://rustup.rs)
- Bitcoin Testnet coins (faucet)
- Electrum server (or change to a public endpoint)

### 📥 Clone & Build

```bash
git clone https://github.com/topnotch1998/rune-etching-rust.git
cd rune-etching-rust
cargo build --release
