# V6 — Architecture Système

## Vue d'ensemble

```
                     ┌─────────────────────────────────────────┐
                     │              GPU Pipeline               │
                     │                                         │
  Trial start ──┬──▶│ Phase 0: Reset activations               │
                │   │   └─ clear first_activation_tick buffer  │
                │   │ Phase 1: Stimulus injection               │
                │   │ Phase 2-4: Zones PID                      │
                │   │ Phase 5: Source contributions              │
                │   │ Phase 6: Propagation (CSR)                │
                │   │ Phase 7: Apply influences + dissipation   │
                │   │   └─ dopa-modulated decay (EF-7)         │
                │   │ ★ Phase 7a: SPARSITY (nouveau V6)        │
                │   │   ├─ Compute scores (activation × bonus) │
                │   │   ├─ Top-K selection (global)            │
                │   │   └─ Suppress non-selected               │
                │   │ Phase 7b: Reverberation                   │
                │   │   └─ dopa-modulated reverb (EF-6)        │
                │   │ Phase 7b': Snapshot activations            │
                │   │ Phase 7c: Adaptive decay                  │
                │   │ Phase 8: Plasticity (STDP 3-factor)       │
                │   │ Phase 9: Budget normalization              │
                │   │ Phase 10: Sync conductances                │
                │   │ Phase 11: Readout                          │
                     └─────────────────────────────────────────┘
```

## Nouveaux fichiers V6

| Fichier | Rôle |
|---------|------|
| `stretch-core/src/sparsity.rs` | Logique CPU : compétition wavefront, suppression |
| `stretch-core/shaders/sparsity.wgsl` | Shader GPU : sparsity pass (scoring + suppression) |
| `configs/config_v6.toml` | Configuration de référence V6 |

## Fichiers modifiés

| Fichier | Modification |
|---------|-------------|
| `stretch-core/src/config.rs` | +V6SparsityConfig, +V6DopaModulationConfig |
| `stretch-core/src/gpu.rs` | +GpuParams V6 fields, +first_activation_tick buffer, +sparsity pipeline |
| `stretch-core/src/simulation.rs` | +sparsity dans step_cpu(), +dopa modulation |
| `stretch-core/shaders/apply_and_dissipate.wgsl` | +update first_activation_tick, +dopa decay mod |
| `stretch-core/shaders/reverberation.wgsl` | +dopa reverb modulation |
| `stretch-core/shaders/gpu_types.wgsl` | (si utilisé) +V6 fields |

## Buffers GPU additionnels

| Buffer | Type | Taille | Binding |
|--------|------|--------|---------|
| `first_activation_tick` | `array<u32>` | num_nodes × 4 bytes | sparsity_bg @binding(1) |

## GpuParams — Champs V6 additionnels

Les 2 champs padding existants (`_pad0`, `_pad1`) + extension à 256 bytes :

| Champ | Type | Offset | Valeur par défaut |
|-------|------|--------|-------------------|
| `sparsity_enabled` | u32 | 224 | 0 |
| `max_active_count` | u32 | 228 | 0 (= num_nodes × fraction) |
| `suppress_factor` | f32 | 232 | 0.0 |
| `novelty_gain` | f32 | 236 | 2.0 |
| `novelty_window` | u32 | 240 | 10 |
| `dopa_mod_enabled` | u32 | 244 | 0 |
| `reverb_min` | f32 | 248 | 0.05 |
| `reverb_max` | f32 | 252 | 0.30 |
| `decay_mod_strength` | f32 | 256 | 0.3 |
| `dopa_threshold` | f32 | 260 | 0.15 |
| `dopa_kappa` | f32 | 264 | 0.05 |
| `_pad_v6` | [u32; 1] | 268 | 0 (→ 272 bytes, 16-aligned) |

**Total : 272 bytes** (vs 224 actuels). 16-byte aligned ✓

## Algorithme de sparsité GPU

Le shader `sparsity.wgsl` opère en deux passes logiques dans un seul dispatch
(car le top-K sur GPU nécessite une approximation) :

### Approche : Seuil adaptatif par histogramme

Plutôt qu'un tri O(n log n) impossible sur GPU, on utilise un **seuil adaptatif** :

1. **Compute scores** : Chaque thread calcule `score_i = activation_i × novelty_bonus_i`
2. **Histogramme atomique** : Distribution des scores dans 64 bins (atomicAdd)
3. **Prefix scan** : Un workgroup unique scanne l'histogramme pour trouver le seuil
   tel que la somme des counts au-dessus = max_active_count
4. **Suppression** : Chaque thread compare son score au seuil, supprime si en-dessous

Ceci se fait en **2 dispatches** (scoring+histogram, puis threshold+suppress), ou
en 3 dispatches (score, scan, suppress) pour la clarté.

### Alternative simple (V6.0) : Seuil approximatif

Pour la première itération, un seuil **précalculé sur CPU** (percentile des
activations du tick précédent) est passé via GpuParams. Plus simple, légèrement
retardé d'un tick.

**Choix V6.0** : Seuil précalculé CPU (simple, efficace). V6.1 pourra passer
au histogramme GPU si nécessaire.

## Modulation dopaminergique

### Dans `apply_and_dissipate.wgsl` :

```wgsl
// EF-7: Dopamine-modulated decay
let dopa = params.dopamine_level;
let sig = 1.0 / (1.0 + exp((dopa - params.dopa_threshold) / params.dopa_kappa));
let decay_mod = 1.0 - params.decay_mod_strength * sig;
effective_decay *= decay_mod;
```

### Dans `reverberation.wgsl` :

```wgsl
// EF-6: Dopamine-modulated reverberation
let sig = 1.0 / (1.0 + exp((dopa - params.dopa_threshold) / params.dopa_kappa));
let reverb_eff = params.reverb_min + (params.reverb_max - params.reverb_min) * sig;
```

Quand dopamine est basse (< θ) : σ → 1 → reverb haute, decay réduit (recherche)
Quand dopamine est haute (> θ) : σ → 0 → reverb basse, decay normal (exploitation)
