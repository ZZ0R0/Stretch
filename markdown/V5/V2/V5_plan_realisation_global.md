# V5 — Plan de réalisation global : Orchestrateur + V5.2

> Document maître ordonnançant l'ensemble des modifications V5.orch et V5.2.
> Chaque phase a ses prérequis, ses livrables, et ses gates de validation.

---

## 1. Vue d'ensemble

### 1.1 Constat

Le code V5.1 a deux problèmes architecturaux :
1. **Duplication structurelle** : `step_gpu()` et `step_cpu()` dupliquent ~92 lignes de logique métier (trial, reward, dopamine).
2. **Divergence CPU/GPU** : les fonctionnalités V5 sustained n'existent que sur CPU ; le GPU tourne en V4 pur.

La V5.2 prévoit d'ajouter du RPE, du port GPU, de l'oubli accéléré — mais chaque ajout aggraverait la duplication si l'architecture reste en V5.1.

### 1.2 Stratégie

**Refactorer AVANT d'ajouter des fonctionnalités.** L'orchestrateur unifié (V5.orch) élimine la duplication et fournit une fondation solide. Les fonctionnalités V5.2 s'ajoutent ensuite proprement, une seule fois chacune.

### 1.3 Séquence complète

```
V5.orch ──→ V5.2a ──→ V5.2b ──→ V5.2c ──→ V5.2 final
   │           │          │          │          │
   │           │          │          │          └── Documentation + archivage
   │           │          │          └── Oubli + Marge + Multi-seed
   │           │          └── Port GPU V5 sustained (shaders)
   │           └── RPE (reward prediction error)
   └── Framework d'exécution unifié (refactoring structurel)
```

---

## 2. Phase 0 : V5.orch — Framework d'exécution unifié

### 2.1 Objectif
Réorganiser `simulation.rs` pour éliminer la duplication CPU/GPU. Zéro changement fonctionnel.

### 2.2 Documentation
| Document | Contenu |
|---|---|
| [V5.orch_cahier_des_charges.md](V5.orch_cahier_des_charges.md) | 21 exigences (ORCH, SEQ, EXT, GPU-B, CPU-B, PERF-O, QUAL-O, COMPAT-O) |
| [V5.orch_architecture_systeme.md](V5.orch_architecture_systeme.md) | Phase enum, TickParams, ComputeBackend trait, GpuBackend, CpuBackend, orchestrateur refactoré |
| [V5.orch_modelisation_mathematique.md](V5.orch_modelisation_mathematique.md) | 7 invariants mathématiques (I1–I7) à préserver |
| [V5.orch_protocoles_evaluation.md](V5.orch_protocoles_evaluation.md) | 9 protocoles (O1–O9) dont 6 bloquants |
| [V5.orch_risques_et_gates.md](V5.orch_risques_et_gates.md) | 5 risques (RO1–RO5), confiance 9/10 |

### 2.3 Étapes d'implémentation

| Étape | Action | Fichiers modifiés | Validation |
|---|---|---|---|
| **0.1** | Enregistrer les métriques de référence V5.1 (CPU + GPU, 4 configs) | — | Métriques sauvegardées |
| **0.2** | Extraire `compute_trial_context()` | `simulation.rs` | `cargo build` + proto O1 |
| **0.3** | Extraire `process_readout()` | `simulation.rs` | `cargo build` + proto O1 |
| **0.4** | Créer `TrialContext`, `TickParams`, `TickResult` structs | `simulation.rs` (ou `backend.rs`) | `cargo build` |
| **0.5** | Créer `build_tick_params()` | `simulation.rs` | `cargo build` |
| **0.6** | Créer `GpuBackend` wrapper | `simulation.rs` + optionnel `backend.rs` | `cargo build` + proto O2 |
| **0.7** | Créer `CpuBackend` | `simulation.rs` + optionnel `backend.rs` | `cargo build` + proto O1 |
| **0.8** | Unifier `step()` → orchestrateur unique, supprimer `step_gpu()`/`step_cpu()` | `simulation.rs` | Proto O1–O5 + O8 |
| **0.9** | Nettoyage : imports, warnings, dead code | tous | `cargo build --release` 0W 0E |

### 2.4 Critères de gate

| Protocole | Statut requis |
|---|---|
| O1 — CPU bit-identical | ✅ PASS |
| O2 — GPU ±0.5% | ✅ PASS |
| O3 — Cohérence CPU↔GPU ≤ V5.1 | ✅ PASS |
| O4 — CPU overhead ≤ 5% | ✅ PASS |
| O5 — GPU overhead ≤ 1% | ✅ PASS |
| O8 — Toutes configs OK | ✅ PASS |

### 2.5 Livrable
- Commit tagué `v5.orch`
- `simulation.rs` refactoré (~325 lignes vs ~400, duplication 0)

---

## 3. Phase 1 : V5.2a — RPE (Reward Prediction Error)

### 3.1 Objectif
Résoudre la saturation du reward. Un seul changement, dans l'orchestrateur.

### 3.2 Prérequis
**V5.orch terminée et validée (gate Phase 0).**

### 3.3 Bénéfice de l'orchestrateur

| Sans orchestrateur (V5.1) | Avec orchestrateur (V5.orch) |
|---|---|
| RPE à implémenter dans `step_gpu()` | RPE implémenté dans `process_readout()` |
| RPE à implémenter dans `step_cpu()` | — (fait une seule fois) |
| δ à passer dans GpuParams manuellement | δ ajouté à TickParams → GpuBackend le mappe |
| 2× risque de divergence | 0 risque de divergence |

### 3.4 Étapes d'implémentation

| Étape | Action | Fichiers modifiés |
|---|---|---|
| **1.1** | Ajouter `rpe_alpha`, `rpe_enabled` dans `RewardConfig` | `config.rs` |
| **1.2** | Ajouter `baseline`, `rpe_delta` dans `RewardSystem` | `reward.rs` |
| **1.3** | Implémenter `compute_rpe()` dans `RewardSystem` | `reward.rs` |
| **1.4** | Appeler `compute_rpe()` dans `process_readout()` (orchestrateur) | `simulation.rs` |
| **1.5** | Faire que la dopamine utilise δ au lieu de r quand RPE activé | `simulation.rs` (process_readout) |
| **1.6** | Ajouter δ, baseline dans `TickParams` pour le GPU | `simulation.rs` ou `backend.rs` |
| **1.7** | Mapper δ → `GpuParams.dopamine` dans `GpuBackend::build_gpu_params()` | `simulation.rs` ou `backend.rs` |
| **1.8** | Exposer δ et baseline dans les métriques | `metrics.rs`, `simulation.rs` |
| **1.9** | Créer `config_v5.2_rpe_*.toml` (4 configs) | `configs/` |

### 3.5 Critères de gate

| Protocole V5.2 | Statut requis |
|---|---|
| 1.1 — RPE convergence | ✅ PASS |
| 1.2 — Matrice 2×2 (Δ_normal ≥ 0, Δ_inv ≥ +77) | ✅ PASS |
| 8 — Non-régression V5.1 (RPE disabled) | ✅ PASS |

### 3.6 Livrable
- Commit tagué `v5.2a`
- RPE fonctionnel sur CPU et GPU

---

## 4. Phase 2 : V5.2b — Port GPU V5 sustained

### 4.1 Objectif
Les fonctionnalités V5 sustained (decay adaptatif, réverbération, reset policy) fonctionnent sur GPU.

### 4.2 Prérequis
**V5.2a terminée et validée (gate Phase 1).**

### 4.3 Bénéfice de l'orchestrateur

| Sans orchestrateur | Avec orchestrateur |
|---|---|
| Ajouter les phases V5 dans `step_gpu()` | Ajouter les phases V5 dans `GpuBackend::execute_tick()` |
| step_cpu() a déjà les phases, mais indépendamment | CpuBackend a déjà les phases (migré depuis step_cpu) |
| Pas de garantie compilateur | Le `Phase` enum force l'implémentation |
| trial/reward/dopamine dupliqués | Pas de duplication |

### 4.4 Étapes d'implémentation

| Étape | Action | Fichiers modifiés |
|---|---|---|
| **2.1** | Étendre `GpuParams` : `reverb_gain`, `k_local`, `reset_policy`, `num_classes`, `rpe_delta`, `reward_baseline`, `rho_boost` | `gpu.rs`, `gpu_types.wgsl` |
| **2.2** | Mapper les champs V5.2 dans `GpuBackend::build_gpu_params()` | `simulation.rs` ou `backend.rs` |
| **2.3** | Créer `reverberation.wgsl` (~30 lignes) | `shaders/` |
| **2.4** | Créer `adaptive_decay.wgsl` (~60 lignes) | `shaders/` |
| **2.5** | Créer `snapshot_activations.wgsl` (~15 lignes) | `shaders/` |
| **2.6** | Ajouter buffer `prev_activations_buf` + CSR outgoing buffers | `gpu.rs` |
| **2.7** | Ajouter les 3 nouveaux pipelines dans `GpuContext` | `gpu.rs` |
| **2.8** | Modifier `run_full_tick()` : insérer reverberation, adaptive_decay, snapshot_activations | `gpu.rs` |
| **2.9** | Modifier `injection.wgsl` : reset policy | `shaders/` |
| **2.10** | Modifier `readout.wgsl` : `num_classes` paramétré | `shaders/` |
| **2.11** | Modifier `plasticity.wgsl` : confirmer que `params.dopamine` = δ fonctionne | `shaders/` |
| **2.12** | Upload CSR outgoing au GPU (dans `GpuContext::try_new()`) | `gpu.rs` |

### 4.5 Critères de gate

| Protocole V5.2 | Statut requis |
|---|---|
| 2.1 — CPU ↔ GPU ±1% | ✅ PASS |
| 2.2 — Sustained ON/OFF comparison | ✅ PASS |
| 2.3 — Mémoire + perf GPU | ✅ PASS |
| 3 — Readout paramétré | ✅ PASS |
| 6.1 — Overhead < 15% | ✅ PASS |

### 4.6 Livrable
- Commit tagué `v5.2b`
- GPU et CPU exécutent le même pipeline V5 + RPE

---

## 5. Phase 3 : V5.2c — Oubli + Marge + Multi-seed

### 5.1 Objectif
Améliorer le remap, réduire la variance, fournir des résultats robustes.

### 5.2 Prérequis
**V5.2b terminée et validée (gate Phase 2).**

### 5.3 Étapes d'implémentation

| Étape | Action | Fichiers modifiés |
|---|---|---|
| **3.1** | Oubli accéléré : $\rho_{\text{eff}} = \rho_0 + \rho_{\text{boost}} \cdot \max(0, -\delta)$ | `stdp.rs` (CPU), `plasticity.wgsl` (GPU), `config.rs` |
| **3.2** | Tester Remap avec RPE seul (proto 4.1) — si > 70%, skip oubli | Test |
| **3.3** | Si nécessaire : activer oubli, tester proto 4.2 | Test |
| **3.4** | Marge (P2) : $r_{\text{eff}} = r \cdot \frac{1}{1 + \beta_M |M|}$ | `simulation.rs` (process_readout) |
| **3.5** | Multi-seed CLI : `--seeds 42,123,456,789,1337` | `stretch-cli/src/main.rs` |
| **3.6** | Export CSV récapitulatif | `stretch-cli/src/main.rs` |

### 5.4 Critères de gate

| Protocole V5.2 | Statut requis |
|---|---|
| 4 — Remap > 70% phase 2 | ✅ PASS |
| 5 — Multi-seed σ < 5 pp | Informatif |
| 7 — Marge réduit σ | Informatif |

### 5.5 Livrable
- Commit tagué `v5.2c`

---

## 6. Phase 4 : V5.2 final — Documentation et archivage

### 6.1 Étapes

| Étape | Action |
|---|---|
| **4.1** | Exécuter la matrice complète finale (4 conditions × 5 seeds + Remap) |
| **4.2** | Rédiger `V5.2_aboutissement.md` avec résultats, graphiques, analyse |
| **4.3** | Mettre à jour `V5.2_etat_des_arts.md` avec l'état post-V5.2 |
| **4.4** | Archiver configs + metrics JSON |
| **4.5** | Commit + tag `v5.2` |

---

## 7. Diagramme de dépendances complet

```
                   ┌────────────────────────────┐
                   │  V5.1 (état actuel)         │
                   │  • step_gpu() / step_cpu()  │
                   │  • ~92 lignes dupliquées    │
                   │  • GPU = V4 seulement       │
                   └──────────┬─────────────────┘
                              │
                   ┌──────────▼─────────────────┐
                   │  Phase 0 : V5.orch          │
                   │  • Orchestrateur unifié      │
                   │  • Phase enum + TickParams   │
                   │  • GpuBackend / CpuBackend   │
                   │  • 0 changement fonctionnel  │
                   │  ─────────────────────────── │
                   │  Gate: O1-O5 + O8           │
                   └──────────┬─────────────────┘
                              │
                   ┌──────────▼─────────────────┐
                   │  Phase 1 : V5.2a — RPE      │
                   │  • δ(t) = r(t) - r̄(t)      │
                   │  • Dans process_readout()    │
                   │    (1 seule implémentation)  │
                   │  ─────────────────────────── │
                   │  Gate: Proto 1.1, 1.2, 8    │
                   └──────────┬─────────────────┘
                              │
                   ┌──────────▼─────────────────┐
                   │  Phase 2 : V5.2b — GPU V5   │
                   │  • 3 nouveaux shaders        │
                   │  • GpuParams 192→224B        │
                   │  • CSR outgoing uploadé      │
                   │  • readout num_classes param  │
                   │  ─────────────────────────── │
                   │  Gate: Proto 2.1-2.3, 3, 6.1│
                   └──────────┬─────────────────┘
                              │
                   ┌──────────▼─────────────────┐
                   │  Phase 3 : V5.2c            │
                   │  • Oubli accéléré (si requis)│
                   │  • Marge (P2)               │
                   │  • Multi-seed CLI           │
                   │  ─────────────────────────── │
                   │  Gate: Proto 4, 5, 7        │
                   └──────────┬─────────────────┘
                              │
                   ┌──────────▼─────────────────┐
                   │  Phase 4 : V5.2 final       │
                   │  • Matrice complète          │
                   │  • Documentation             │
                   │  • Tag v5.2                  │
                   └────────────────────────────┘
```

---

## 8. Matrice impact × fichier

| Fichier | V5.orch | V5.2a | V5.2b | V5.2c |
|---|---|---|---|---|
| `simulation.rs` | ★★★ Refactoring majeur | ★★ process_readout + RPE | ★ build_gpu_params | ★ marge dans process_readout |
| `gpu.rs` | — | — | ★★★ Pipelines + buffers | — |
| `config.rs` | — | ★ rpe_alpha, rpe_enabled | — | ★ rho_boost, margin_beta |
| `reward.rs` | — | ★★ baseline, rpe_delta, compute_rpe() | — | — |
| `stdp.rs` | — | — | — | ★ oubli accéléré ρ_eff |
| `dopamine.rs` | — | ★ δ au lieu de r | — | — |
| `metrics.rs` | — | ★ exposer δ, baseline | — | — |
| `gpu_types.wgsl` | — | — | ★ GpuParams étendu | — |
| `injection.wgsl` | — | — | ★ reset_policy | — |
| `readout.wgsl` | — | — | ★ num_classes | — |
| `plasticity.wgsl` | — | — | ★ (confirm δ OK) | ★ rho_boost |
| Nouveaux shaders | — | — | ★★ reverberation, adaptive_decay, snapshot | — |
| `stretch-cli` | — | — | — | ★★ multi-seed |

---

## 9. Estimation volume de code

| Phase | Lignes modifiées | Lignes nouvelles | Lignes supprimées | Net |
|---|---|---|---|---|
| V5.orch | ~200 restructurées | ~250 (backend, TickParams, etc.) | ~200 (step_gpu, step_cpu) | **~+50** |
| V5.2a | ~30 | ~60 (RPE logic, configs) | ~0 | **+60** |
| V5.2b | ~80 (gpu.rs) | ~200 (shaders, buffers, pipelines) | ~0 | **+200** |
| V5.2c | ~30 | ~100 (oubli, marge, multi-seed, CSV) | ~0 | **+100** |
| **Total** | **~340** | **~610** | **~200** | **+410** |

---

## 10. Critère de réussite global V5.2

La V5.2 est un succès si et seulement si :

| # | Critère | Source |
|---|---|---|
| 1 | Orchestrateur unifié, 0 duplication | Code review |
| 2 | RPE résout Δ_normal ≥ 0 (plus de dégradation) | Benchmark |
| 3 | GPU et CPU exécutent le même pipeline V5 + RPE | Benchmark |
| 4 | Overhead GPU < 15% | Profiling |
| 5 | Remap > 70% en phase 2 | Benchmark |
| 6 | Résultats multi-seed robustes (σ < 5 pp) | Benchmark |

**La matrice cible finale :**

| | TopologyOnly | FullLearning | Δ |
|---|---|---|---|
| **Normal** | ~97% | **≥ 97%** | **≥ 0 pp** |
| **Inversé** | ~3% | **≥ 80%** | **≥ +77 pp** |
| **Remap phase 2** | — | **> 70%** | — |
