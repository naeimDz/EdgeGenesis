# EdgeGenesis: Evolutionary Edge Computing Laboratory

> **"We are not building a game; we are building a laboratory."**

EdgeGenesis is a high-fidelity scientific simulation built with **Rust** and the **Bevy Engine**. It simulates a network of solar-powered Edge AI devices evolving under harsh environmental constraints to discover optimal resource management algorithms.

---

## 🔬 Scientific Background

The proliferation of Edge AI is constrained by two critical factors: **Energy Availability** and **Model Efficiency**. 
This laboratory simulates a "Digital Darwinism" scenario where devices must autonomously balance:
*   **Inference Accuracy:** Running complex models (Transformers/CNNs) improves utility but drains power.
*   **Energy Survival:** Solar harvesting is intermittent and scarce. Batteries are finite.
*   **Evolutionary Pressure:** Only nodes that sustain operations and perform useful work pass their configuration (genes) to the next generation.

---

## 🛠 Application Architecture: Hybrid Data System

The system implements a **Hybrid Data Architecture** that combines scientific rigor with experimental flexibility.

### 1. The Foundation (`models.rs`)
*   **Role:** Single source of truth for model specifications.
*   **Nature:** Type-safe, compiled, and scientifically documented.
*   **Data:** Contains hardcoded, verified benchmarks (e.g., Pi 4 power profiles).

### 2. The Override Layer (`data_loader.rs` + CSV)
*   **Role:** Optional runtime customization.
*   **Nature:** Flexible and experimental.
*   **Data:** CSV files in `data/` can override specific values without recompilation.

### 3. The ECS Core (`systems.rs` + `components.rs`)
*   **Entities:** Edge Nodes.
*   **Components:** Pure data structs (`Battery`, `Gene`) populated from the hybrid layer.
*   **Systems:** Logic execution (`resource_physics`) oblivious to the data source.

See [ARCHITECTURE.md](ARCHITECTURE.md) for a deep dive.

---

## ✅ Completed Milestones

### 1. Core Physics Engine
- [x] **Real-World Data Integration:**
    - Integrated `raspberry_pi_4.csv` profile (Idle: 2.5W, Inference: 3.5W-6.2W).
    - Integrated `algiers_solar.csv` for realistic diurnal solar cycles.
- [x] **Energy Dynamics:**
    - Accurate Battery Drain = `(Base_Load + Inference_Load) * Time`.
    - Solar Harvesting = `Irradiance * Panel_Efficiency * Cloud_Factor`.

### 2. Evolutionary Biology
- [x] **Genomic Structure:** Genes controlling Model Selection (Nano to TinyBERT) and duty cycles.
- [x] **Natural Selection:** "Survival of the Fittest" - nodes dying from energy depletion are culled.
- [x] **Mutation Engine:** Random variations in solar efficiency and inference frequency introduce diversity.

### 3. Visualization (Sci-Fi HUD)
- [x] **Minimalist UI:** Data-first visualization using Bevy Gizmos.
- [x] **State Indicators:** Color-coded battery levels (Green → Yellow → Red → Gray/Dead).
- [x] **Spatial Grid:** 10x10 distributed sensor network layout.

---

## 📊 Data Verification & Gap Analysis

Our simulation strength relies on the fidelity of input data.

### Currently Used Data Sources
| Dataset | Source | Usage in Sim |
| :--- | :--- | :--- |
| **Power Profiles** | Benchmarks (Raspberry Pi 4) | Determining `idle_power_w` and `inference_power_w` |
| **Solar Data** | Typical Meteorological Year (Algiers) | `avg_irradiance` drives energy harvesting logic |
| **Model Metrics** | PapersWithCode / HuggingFace | `accuracy_percent` used for fitness evaluation |

### Unused Data & Improvement Opportunities
We achieved **~70% data utilization**. The following attributes exist in our CSVs but are currently dormant:

1.  **`avg_inference_time_ms` (Latency):**
    *   *Current:* Simulation runs on fixed time-steps.
    *   *Improvement:* Use latency to calculate max theoretical throughput per second (FPS limit).
2.  **`model_size_mb` (Memory Footprint):**
    *   *Current:* All Nodes are assumed to have infinite RAM.
    *   *Improvement:* Introduce hardware tiers (e.g., Pi Zero vs Pi 4). Nodes with low RAM should die or crash if they evolve to pick large models (e.g., TinyBERT on 512MB RAM).
3.  **`parameters_millions` (Complexity):**
    *   *Current:* Proxy for accuracy.
    *   *Improvement:* Use as a factor for "Upgrade Cost" or evolutionary energy penalty during mutation.

---

## 🧪 Experiment Status: "Nuclear Winter"

We are currently running a **High-Stress Survival Scenario**:
*   **Battery Capacity:** Reduced to **0.5 Wh** (Micro-capacitors).
*   **Solar Availability:** **0%** (Total Darkness / System Shock).
*   **Base Load:** **20W** (Extreme computational load).

**Objective:** Force rapid evolutionary convergence. In this harsh environment, only the most hyper-efficient configurations should survive more than a few seconds, allowing us to observe rapid generation turnover.

---

## 🚀 Roadmap & Future Steps

### Phase 1: Environmental Complexity (Next)
- [ ] Implement **Dynamic Weather Patterns** (passing clouds, seasonal shifts).
- [ ] Add **Thermal Throttling**: CPU temperature affecting inference speed/power.

### Phase 2: Neural Evolution
- [ ] **Brain Gene:** Evolve a small Neural Network (weights) to make runtime decisions (sleep vs. work) instead of static frequencies.
- [ ] **Peer-to-Peer Offloading:** Allow dying nodes to offload tasks to neighbors with surplus energy.

### Phase 3: Data Analysis
- [ ] **Heatmaps:** Export long-term survival heatmaps to CSV/JSON.
- [ ] **Genealogy Tracking:** Visualizing the "Family Tree" of the most successful algorithm.

---

## 💻 Running the Simulation

**Prerequisites:** Rust & Cargo installed.

```bash
# Clone and Setup
git clone [repo-url]
cd EdgeGenesis

# Run Simulation (Hot Reload Enabled)
# Warning: Current presets are FATAL for rapid testing.
cargo run
```

## 📂 Project Structure

```
src/
├── main.rs              # Entry point - loads CSV overrides, wires systems
├── models.rs            # ⭐ PRIMARY DATA SOURCE (Type-safe, documented)
├── data_loader.rs       # CSV loader + hybrid helper functions
├── components.rs        # Pure ECS data structures (no logic)
└── systems.rs           # Business logic (physics, evolution, rendering)
data/
├── power_profiles/      # Hardware Benchmarks (CSV overrides)
└── solar_profiles/      # Weather Datasets
```
