# V4_corrections_3 — Architecture GPU-First & Optimisations Performance

> **Objectif** : Transformer la pipeline de simulation d'un modèle hybride CPU↔GPU
> en une architecture **GPU-first** où l'intégralité du tick de simulation
> s'exécute sur le GPU, sans aller-retour CPU↔GPU par tick.
> Le CPU ne sert plus qu'à orchestrer les trials, injecter les stimuli
> (quelques dizaines de nœuds) et lire les readouts (quelques dizaines de valeurs).
>
> Cible : tick < 3ms (vs 29ms aujourd'hui) → >300 ticks/sec.

---

## Table des matières

- [PARTIE A — Diagnostic de l'architecture actuelle](#partie-a--diagnostic-de-larchitecture-actuelle)
- [PARTIE B — Architecture GPU-First](#partie-b--architecture-gpu-first)
- [PARTIE C — Nouveaux shaders WGSL](#partie-c--nouveaux-shaders-wgsl)
- [PARTIE D — Nouveau GpuContext](#partie-d--nouveau-gpucontext)
- [PARTIE E — Nouveau Simulation::step()](#partie-e--nouveau-simulationstep)
- [PARTIE F — Optimisations de la visualisation](#partie-f--optimisations-de-la-visualisation)
- [PARTIE G — Optimisations additionnelles](#partie-g--optimisations-additionnelles)
- [PARTIE H — Checklist d'implémentation](#partie-h--checklist-dimplémentation)

---

## PARTIE A — Diagnostic de l'architecture actuelle

### A.1 Profiling mesuré (50k nœuds, 577k arêtes)

```
[PERF ticks 1200-1300] (min/avg/max ms)
  zones:       1.26 / 1.74 / 2.20
  stim+input:  0.00 / 0.02 / 0.67
  PROPAG:      4.48 / 5.30 / 7.64
  dissip:      0.50 / 0.66 / 1.08
  PLAST+STDP: 19.54 / 21.42 / 25.55
  readout+rew: 0.00 / 0.00 / 0.10
  metrics:     0.00 / 0.24 / 2.88
  ─────────────────────────────────
  total_avg:  29.36ms (~34 ticks/sec)
```

**73% du temps** est dans PLAST+STDP. Voyons pourquoi.

### A.2 Anatomie du goulot d'étranglement PLAST+STDP (path GPU)

Sur le chemin GPU dans `simulation.rs` lignes 400-495, chaque tick exécute
**séquentiellement** :

```
  CPU: par_iter 50k nodes → update last_activation_tick    ~0.5ms
  CPU: snapshot_activation_ticks (copie 50k Option<usize>)  ~0.3ms
  CPU→GPU: upload activation_ticks (50k × f64→i32)          ~0.2ms
  CPU→GPU: upload node_delta_dopa  (50k × f64→f32)          ~0.2ms
  CPU→GPU: upload params uniform                             ~0.0ms
  GPU: run_plasticity()                                      ~2ms compute
  CPU: device.poll(Wait) ← STALL BLOQUANT #1                ~2-3ms
  GPU: run_budget_norm() (clear + 2 passes)                  ~1ms compute
  CPU: device.poll(Wait) ← STALL BLOQUANT #2                ~2-3ms
  GPU→CPU: download_edges (577k×32B = 18.5 MB)              ~5-6ms
  CPU: device.poll(Wait) + map_async ← STALL BLOQUANT #3    ~2-3ms
  CPU: deserialize 577k GpuEdge → domain.edges               ~1.5ms
  CPU→GPU: upload_conductances (reorder 577k f64→f32 CSR)    ~1ms
  CPU: sync_conductances (copie 577k edge.cond → vec)        ~0.5ms
```

**Total : ~21ms** dont :
- **~7-9ms de stalls CPU** (3 × `device.poll(Maintain::Wait)`)
- **~6ms de transfert GPU→CPU** (18.5 MB à chaque tick)
- **~3ms de conversions f64↔f32** sur 577k éléments
- **~3ms de compute GPU réel** (plasticity + budget)

### A.3 Pipeline actuelle vs pipeline cible

```
┌─── ARCHITECTURE ACTUELLE (V4.2) ────────────────────────────────────────┐
│                                                                          │
│  ┌─────────┐     ┌──────────┐     ┌─────────┐     ┌──────────┐         │
│  │  ZONES  │────▶│ STIMULUS │────▶│ PROPAG  │────▶│ DISSIP   │         │
│  │  (CPU)  │     │  (CPU)   │     │GPU+CPU  │     │  (CPU)   │         │
│  └─────────┘     └──────────┘     └────┬────┘     └──────────┘         │
│                                      ▼  ▲                                │
│                                 ╔═════════════╗                          │
│                                 ║  GPU↔CPU    ║  ← round-trip           │
│                                 ║  download   ║     influence           │
│                                 ╚═════════════╝                          │
│                                        │                                 │
│  ┌──────────────────────────────────────┘                                │
│  ▼                                                                       │
│  ┌──────────┐     ╔═══════════════╗     ╔═══════════════╗               │
│  │ PLAST    │────▶║   UPLOAD      ║────▶║  GPU compute  ║               │
│  │ prep CPU │     ║ ticks+dopa    ║     ║  plasticity   ║               │
│  └──────────┘     ╚═══════════════╝     ╚══════╤════════╝               │
│                                                  │                       │
│                   ╔═══════════════╗     ╔════════╧════════╗             │
│                   ║   DOWNLOAD    ║◀───║  GPU budget     ║             │
│                   ║  ALL edges    ║     ║  (2 passes)    ║             │
│                   ║  18.5 MB !!   ║     ╚════════════════╝             │
│                   ╚══════╤════════╝                                     │
│                          │                                               │
│  ┌───────────────────────┘                                              │
│  ▼                                                                       │
│  ┌──────────┐     ╔═══════════════╗     ┌──────────┐                   │
│  │ deser.   │────▶║  RE-UPLOAD    ║────▶│ READOUT  │                   │
│  │ edges    │     ║  conductances ║     │  (CPU)   │                   │
│  │ (CPU)    │     ╚═══════════════╝     └──────────┘                   │
│  └──────────┘                                                            │
│                                                                          │
│  3 stalls GPU bloquants × ~2-3ms = 7-9ms perdus                        │
│  18.5 MB transférés GPU→CPU + re-upload CPU→GPU chaque tick             │
└──────────────────────────────────────────────────────────────────────────┘
```

### A.4 Pourquoi l'état vit côté CPU (racine du problème)

L'état des nœuds (`Node` struct) est côté CPU uniquement :
- `activation` (f64)
- `threshold`, `fatigue`, `memory_trace`, `excitability` (f64)
- `inhibition`, `threshold_mod` (f64)
- `last_activation_tick` (Option<usize>)
- `node_type` (enum)

Chaque phase du tick lit/écrit ces champs :
- **Zones** : lit `activation`, écrit `threshold_mod`
- **Stimulus** : écrit `activation`
- **Propagation** : lit `activation`, `threshold`, écrit `activation`
- **Dissipation** : lit/écrit `activation`, `fatigue`, `inhibition`, `memory_trace`

→ L'état-nœud est partagé par toutes les phases → toutes doivent tourner
côté CPU ou être portées **ensemble** sur GPU.

---

## PARTIE B — Architecture GPU-First

### B.1 Principe fondamental

> **Tout l'état mutable (nœuds + arêtes) vit sur le GPU.**
> Le CPU n'orchestre que la logique de haut niveau (trials, reward).
> Un seul `queue.submit()` par tick, zéro readback sauf aux points de décision.

### B.2 Schéma de la nouvelle pipeline

```
┌─── ARCHITECTURE GPU-FIRST (V4.3) ───────────────────────────────────────┐
│                                                                          │
│  CPU (orchestrateur)          GPU (compute)                              │
│  ┌────────────────┐          ┌─────────────────────────────────────┐    │
│  │                │          │                                     │    │
│  │ Trial logic:   │  PETIT   │  ┌──────────┐     ┌──────────┐    │    │
│  │ - schedule     │ upload   │  │ INJECTION│────▶│  ZONES   │    │    │
│  │ - reward calc  │────────▶│  │  shader  │     │  shader  │    │    │
│  │ - dopamine     │(~200 B)  │  └──────────┘     └──────────┘    │    │
│  │                │          │        │                  │         │    │
│  │                │          │        ▼                  ▼         │    │
│  │                │          │  ┌──────────┐     ┌──────────┐    │    │
│  │                │          │  │  SOURCE  │────▶│  PROPAG  │    │    │
│  │                │          │  │  CONTRIBS│     │  CSR     │    │    │
│  │                │          │  │  shader  │     │  shader  │    │    │
│  │                │          │  └──────────┘     └──────────┘    │    │
│  │                │          │                          │         │    │
│  │                │          │                          ▼         │    │
│  │                │          │  ┌──────────┐     ┌──────────┐    │    │
│  │                │          │  │  APPLY   │────▶│ DISSIP   │    │    │
│  │                │          │  │  INFLUEN │     │  shader  │    │    │
│  │                │          │  └──────────┘     └──────────┘    │    │
│  │                │          │                          │         │    │
│  │                │          │                          ▼         │    │
│  │                │          │  ┌──────────┐     ┌──────────┐    │    │
│  │                │          │  │ PLAST  + │────▶│  BUDGET  │    │    │
│  │                │          │  │ STDP     │     │  NORM    │    │    │
│  │                │          │  │  shader  │     │  shader  │    │    │
│  │                │          │  └──────────┘     └──────────┘    │    │
│  │                │          │                                     │    │
│  │                │  PETIT   │       TOUT DANS UN SEUL             │    │
│  │ Readout ◀─────│◀─────── │       CommandEncoder                │    │
│  │ (~400 B)      │ download │       → 1 submit / tick             │    │
│  │               │          │       → 0 stall intermédiaire       │    │
│  └────────────────┘          └─────────────────────────────────────┘    │
│                                                                          │
│  Transferts par tick :                                                   │
│    CPU→GPU : params uniform (~128 B) + stimulus mask (~200 B)           │
│    GPU→CPU : readout scores (~64 B), seulement aux ticks de décision    │
│                                                                          │
│  vs architecture actuelle : 18.5 MB GPU→CPU + 2.3 MB CPU→GPU / tick    │
└──────────────────────────────────────────────────────────────────────────┘
```

### B.3 Buffers GPU persistants — nouveau layout

L'état complet du réseau vit sur le GPU dans des buffers structurés :

#### B.3.1 Buffer `buf_nodes` — état des nœuds (NOUVEAU)

```wgsl
struct GpuNode {
    activation: f32,           // offset 0
    threshold: f32,            // offset 4
    fatigue: f32,              // offset 8
    memory_trace: f32,         // offset 12
    excitability: f32,         // offset 16
    inhibition: f32,           // offset 20
    threshold_mod: f32,        // offset 24  (PID zones)
    last_activation_tick: i32, // offset 28  (-1 = jamais)
    activation_count: u32,     // offset 32
    is_excitatory: u32,        // offset 36  (0 = inhib, 1 = excit)
    gain_mod: f32,             // offset 40  (PID zones)
    _pad: f32,                 // offset 44  → 48 bytes, aligné 16
};
```

**Taille** : 50k × 48 B = **2.4 MB** — réside en permanence sur le GPU.

#### B.3.2 Buffer `buf_edges` — état des arêtes (existant, conservé)

```wgsl
struct GpuEdge {
    from_node: u32,
    to_node: u32,
    conductance: f32,
    eligibility: f32,
    consolidated: u32,
    consolidation_counter: u32,
    distance: f32,
    _pad: f32,
};
```

**Taille** : 577k × 32 B = **18.5 MB** — réside en permanence sur le GPU.

#### B.3.3 Buffer `buf_conductances` — cache CSR (existant, conservé)

Les conductances en ordre CSR pour la propagation.
**Important** : ce buffer est maintenant mis à jour par un **shader** après
la plasticité, au lieu d'un round-trip CPU.

**Taille** : 577k × 4 B = **2.3 MB**.

#### B.3.4 Buffer `buf_source_contribs` — contributions sources (NOUVEAU GPU)

Calculé par le nouveau shader `source_contribs.wgsl`.
**Taille** : 50k × 4 B = **200 KB**.

#### B.3.5 Buffer `buf_influences` — influences entrantes (existant)

**Taille** : 50k × 4 B = **200 KB**.

#### B.3.6 Buffer `buf_params` — paramètres uniformes (existant, étendu)

```wgsl
struct GpuParams {
    // --- Tailles ---
    num_nodes: u32,
    num_edges: u32,
    current_tick: u32,
    // --- Propagation ---
    propagation_gain: f32,
    gain_inhibitory: f32,
    // --- Dissipation ---
    activation_decay: f32,
    activation_min: f32,
    fatigue_gain: f32,
    fatigue_recovery: f32,
    inhibition_gain: f32,
    inhibition_decay: f32,
    trace_gain: f32,
    trace_decay: f32,
    decay_jitter: f32,
    // --- STDP / Plasticité ---
    a_plus: f32,
    a_minus: f32,
    tau_plus: f32,
    tau_minus: f32,
    elig_decay: f32,
    elig_max: f32,
    plasticity_gain: f32,
    global_delta_dopa: f32,
    dopa_phasic: f32,
    use_spatial: u32,
    spatial_lambda: f32,
    cond_min: f32,
    cond_max: f32,
    homeostatic_rate: f32,
    baseline_cond: f32,
    dopamine_level: f32,
    dopa_consol_threshold: f32,
    consol_conductance_threshold: f32,
    consol_ticks_required: u32,
    budget: f32,
    // --- Stimulus ---
    stimulus_class: i32,       // -1 = pas de stimulus ce tick
    stimulus_intensity: f32,
    // --- Padding ---
    _pad0: f32,
    _pad1: f32,
};
```

**Taille** : ~144 B — uploadé une fois par tick (trivial).

#### B.3.7 Buffer `buf_stimulus_mask` — masque de stimulus (NOUVEAU)

Petit buffer de bits ou d'indices : quels nœuds reçoivent le stimulus.
Pré-uploadé une fois au setup (les groupes d'entrée sont fixes).

```
stimulus_groups[class_id × group_size + k] = node_index
```

**Taille** : 2 classes × 50 nœuds × 4 B = **400 B** (uploadé une seule fois).

#### B.3.8 Buffer `buf_zone_assignments` — zone par nœud (NOUVEAU)

```
zone_assignments[node_i] = zone_id   (u32)
```

Pré-uploadé une fois au setup.
**Taille** : 50k × 4 B = **200 KB** (uploadé une seule fois).

#### B.3.9 Buffer `buf_zone_state` — état PID des zones (NOUVEAU)

```wgsl
struct GpuZone {
    target_activity: f32,
    activity_sum: f32,      // accumulé par reduce
    member_count: u32,
    error: f32,
    integral: f32,
    error_prev: f32,
    output: f32,
    theta_mod: f32,
    gain_mod: f32,
    stable_ticks: u32,
    is_stable: u32,
    _pad: f32,              // → 48 bytes
};
```

**Taille** : 8 zones × 48 B = **384 B**.

#### B.3.10 Buffer `buf_readout_scores` — scores de sortie (NOUVEAU)

```
readout_scores[class_id] = f32  (somme des activations du groupe)
```

**Taille** : 2 classes × 4 B = **8 B**. C'est le **seul** readback GPU→CPU,
et uniquement aux ticks de décision (~1 tous les 31 ticks).

#### B.3.11 Buffer `buf_readout_groups` — groupes de sortie (NOUVEAU)

Pré-uploadé au setup.

```
readout_groups[class_id × group_size + k] = node_index
```

**Taille** : 2 × 50 × 4 B = **400 B** (uploadé une seule fois).

### B.4 Résumé des transferts CPU↔GPU par tick

| Direction | Données | Taille | Fréquence |
|-----------|---------|--------|-----------|
| CPU→GPU | `buf_params` (uniform) | 144 B | Chaque tick |
| GPU→CPU | `buf_readout_scores` | 8 B | 1x / 31 ticks (aux read_ticks) |
| **Total** | | **~150 B/tick** | |

**vs actuel** : ~20.8 MB/tick (18.5 MB edges download + 2.3 MB conductances upload).

**Facteur d'amélioration transferts : ~140 000×.**

---

## PARTIE C — Nouveaux shaders WGSL

### C.1 `injection.wgsl` — Injection de stimulus (NOUVEAU)

Remplace `InputEncoder::inject()` (CPU) + `stimulus::inject_stimuli()` (CPU).

```wgsl
// injection.wgsl — inject stimulus into input group nodes
// 1 thread per node in the stimulus group (petit dispatch ~50 threads)

@group(0) @binding(0) var<storage, read_write> nodes: array<GpuNode>;
@group(0) @binding(1) var<storage, read>       stimulus_groups: array<u32>;
@group(0) @binding(2) var<uniform>             params: GpuParams;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let class = params.stimulus_class;
    if (class < 0) { return; }

    let group_size = 50u;  // paramétrisable via params si besoin
    let base = u32(class) * group_size;
    let k = gid.x;
    if (k >= group_size) { return; }

    let node_idx = stimulus_groups[base + k];
    nodes[node_idx].activation += params.stimulus_intensity;
    nodes[node_idx].activation = min(nodes[node_idx].activation, 10.0);
}
```

**Coût** : ~50 threads → quasi-gratuit.

### C.2 `zones.wgsl` — Mesure + PID zones (NOUVEAU)

Deux passes :

**Passe 1 — `zone_measure`** : chaque thread = 1 nœud, ajoute son activation
au compteur atomique de sa zone.

**Passe 2 — `zone_regulate`** : chaque thread = 1 zone, calcule le PID et
écrit theta_mod/gain_mod. Puis un 3e dispatch (1 thread/nœud) applique
theta_mod/gain_mod de la zone aux nœuds membres.

```wgsl
// zones.wgsl — zone PID on GPU

struct GpuNode { /* ... comme B.3.1 ... */ };
struct GpuZone { /* ... comme B.3.9 ... */ };

@group(0) @binding(0) var<storage, read_write> nodes: array<GpuNode>;
@group(0) @binding(1) var<storage, read_write> zones: array<GpuZone>;
@group(0) @binding(2) var<storage, read>       zone_assignments: array<u32>;
@group(0) @binding(3) var<uniform>             params: GpuParams;

// --- Passe 1 : accumulation atomique des activations par zone ---
@compute @workgroup_size(256)
fn zone_measure(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= params.num_nodes) { return; }

    let zone_id = zone_assignments[idx];
    let activation = nodes[idx].activation;

    // Atomique fixed-point (activation × 1000000 → u32)
    let act_fixed = u32(activation * 1000000.0);
    atomicAdd(&zone_activity_accum[zone_id], act_fixed);
    // Note : zone_activity_accum est un buffer auxiliaire atomic<u32>
    // qui est clear à 0 avant chaque tick (dans le même encoder)
}

// --- Passe 2 : PID par zone (1 thread par zone, petit dispatch) ---
@compute @workgroup_size(8)
fn zone_pid(@builtin(global_invocation_id) gid: vec3<u32>) {
    let z = gid.x;
    if (z >= num_zones) { return; }

    let sum_fixed = atomicLoad(&zone_activity_accum[z]);
    let mean = f32(sum_fixed) / 1000000.0 / f32(zones[z].member_count);
    zones[z].activity_sum = mean;

    // Stability check: skip si stable (C6)
    if (zones[z].is_stable != 0u && zones[z].stable_ticks % 10u != 0u) {
        zones[z].stable_ticks += 1u;
        return;
    }

    // PID
    let error = zones[z].target_activity - mean;
    let integral = clamp(zones[z].integral + error, -pid_integral_max, pid_integral_max);
    let derivative = error - zones[z].error_prev;
    let u = clamp(kp * error + ki * integral + kd * derivative, -pid_out_max, pid_out_max);

    zones[z].error_prev = error;
    zones[z].error = error;
    zones[z].integral = integral;
    zones[z].output = u;

    // Indirect mode: theta_mod, gain_mod
    zones[z].theta_mod = -k_theta * u;
    zones[z].gain_mod = k_gain * u;

    // Stability tracking (C6)
    if (abs(error) < 0.01) {
        zones[z].stable_ticks += 1u;
        if (zones[z].stable_ticks >= 50u) {
            zones[z].is_stable = 1u;
        }
    } else {
        zones[z].stable_ticks = 0u;
        zones[z].is_stable = 0u;
    }
}

// --- Passe 3 : appliquer theta_mod et gain_mod aux nœuds (1 thread/nœud) ---
@compute @workgroup_size(256)
fn zone_apply(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= params.num_nodes) { return; }

    let z = zone_assignments[idx];
    nodes[idx].threshold_mod = zones[z].theta_mod;
    nodes[idx].gain_mod = zones[z].gain_mod;
}
```

**Coût estimé** : 3 dispatches minuscules → ~0.2ms total.

### C.3 `source_contribs.wgsl` — Calcul des contributions source (NOUVEAU)

Remplace `propagation::compute_source_contribs()` (CPU).

```wgsl
// source_contribs.wgsl — 1 thread per node
// Calcule source_contrib[i] = activation[i] × sign × gain × gain_mod
// si nœud actif, sinon 0

@group(0) @binding(0) var<storage, read>       nodes: array<GpuNode>;
@group(0) @binding(1) var<storage, read_write> source_contribs: array<f32>;
@group(0) @binding(2) var<uniform>             params: GpuParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= params.num_nodes) { return; }

    let node = nodes[idx];

    // is_active = activation > effective_threshold
    let eff_threshold = max(
        (node.threshold + node.fatigue + node.inhibition + node.threshold_mod)
        / max(node.excitability, 0.01),
        0.05
    );

    if (node.activation <= eff_threshold) {
        source_contribs[idx] = 0.0;
        return;
    }

    let sign = select(-params.gain_inhibitory, 1.0, node.is_excitatory != 0u);
    let gain_mod = 1.0 + node.gain_mod;

    source_contribs[idx] = node.activation * sign * gain_mod * params.propagation_gain;
}
```

**Coût** : 50k threads → ~0.1ms.

### C.4 `propagation.wgsl` — Propagation CSR (existant, conservé tel quel)

Lit `source_contribs` + `conductances` + CSR → écrit `influences`.
**Aucun changement** nécessaire.

### C.5 `apply_and_dissipate.wgsl` — Application influences + dissipation (NOUVEAU)

Fusionne `apply_influences()` + dissipation en **un seul shader**.
Remplace les deux passes CPU séparées.

```wgsl
// apply_and_dissipate.wgsl — 1 thread per node
// Applique l'influence entrante, puis la dissipation complète

@group(0) @binding(0) var<storage, read_write> nodes: array<GpuNode>;
@group(0) @binding(1) var<storage, read>       influences: array<f32>;
@group(0) @binding(2) var<uniform>             params: GpuParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= params.num_nodes) { return; }

    var node = nodes[idx];

    // --- 1. Appliquer l'influence ---
    let infl = influences[idx];
    node.activation += infl;
    node.activation = max(node.activation, 0.0);
    node.activation = min(node.activation, 10.0);

    // --- 2. Vérifier si actif (pour STDP tick update) ---
    let eff_threshold = max(
        (node.threshold + node.fatigue + node.inhibition + node.threshold_mod)
        / max(node.excitability, 0.01),
        0.05
    );
    let was_active = node.activation > eff_threshold;

    if (was_active) {
        node.last_activation_tick = i32(params.current_tick);
        node.activation_count += 1u;
    }

    // --- 3. Dissipation ---
    // Decay jitter basé sur hash du node index + tick
    var effective_decay = params.activation_decay;
    if (params.decay_jitter > 0.0) {
        // Simple hash pour jitter reproductible
        let h = (u32(idx) * 6364136223u + params.current_tick * 1442695040u);
        let jitter_val = (f32(h >> 16u) / 65536.0 - 0.5) * 2.0 * params.decay_jitter;
        effective_decay = clamp(effective_decay * (1.0 + jitter_val), 0.0, 1.0);
    }

    // Fatigue
    if (was_active) {
        node.fatigue += params.fatigue_gain * node.activation;
    }
    node.fatigue = clamp(node.fatigue * (1.0 - params.fatigue_recovery), 0.0, 10.0);

    // Inhibition
    if (was_active) {
        node.inhibition += params.inhibition_gain;
    }
    node.inhibition = clamp(node.inhibition * (1.0 - params.inhibition_decay), 0.0, 10.0);

    // Trace mémoire
    if (was_active) {
        node.memory_trace += params.trace_gain * node.activation;
    }
    node.memory_trace = clamp(node.memory_trace * (1.0 - params.trace_decay), 0.0, 100.0);

    // Excitability from trace
    node.excitability = 1.0 + 0.1 * node.memory_trace;

    // Activation decay
    node.activation *= (1.0 - effective_decay);
    node.activation = max(node.activation, params.activation_min);

    // --- 4. Écrire l'état mis à jour ---
    nodes[idx] = node;
}
```

**Coût** : 50k threads, ~0.2ms.

### C.6 `plasticity.wgsl` — STDP + 3-facteur (existant, conservé)

Le shader actuel fonctionne bien. Il lit `activation_ticks` du buffer
`buf_nodes` (champ `last_activation_tick`) au lieu d'un buffer séparé.

**Modification** : lire les activation ticks directement depuis `buf_nodes`
au lieu d'un buffer uploadé.

```wgsl
// Modification dans plasticity.wgsl :
// Remplacer :
//   @group(0) @binding(1) var<storage, read> activation_ticks: array<i32>;
// Par :
//   @group(0) @binding(1) var<storage, read> nodes: array<GpuNode>;
// Et lire :
//   let t_pre = nodes[src].last_activation_tick;
//   let t_post = nodes[dst].last_activation_tick;
```

### C.7 `sync_conductances.wgsl` — Sync GPU→GPU (NOUVEAU)

Remplace le round-trip CPU (download edges → reorder → upload conductances).

```wgsl
// sync_conductances.wgsl — 1 thread per CSR entry
// Copie edges[csr_edge_indices[k]].conductance → conductances[k]

@group(0) @binding(0) var<storage, read>       edges: array<GpuEdge>;
@group(0) @binding(1) var<storage, read>       csr_edge_indices: array<u32>;
@group(0) @binding(2) var<storage, read_write> conductances: array<f32>;
@group(0) @binding(3) var<uniform>             params: GpuParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let k = gid.x;
    if (k >= params.num_edges) { return; }

    let edge_idx = csr_edge_indices[k];
    conductances[k] = edges[edge_idx].conductance;
}
```

**Coût** : 577k threads → ~0.5ms.
**Gain** : élimine le download 18.5 MB + conversion + re-upload 2.3 MB.

### C.8 `readout.wgsl` — Lecture de sortie (NOUVEAU)

```wgsl
// readout.wgsl — accumule les activations par groupe de sortie
// 1 thread par nœud de sortie

@group(0) @binding(0) var<storage, read>       nodes: array<GpuNode>;
@group(0) @binding(1) var<storage, read>       readout_groups: array<u32>;
@group(0) @binding(2) var<storage, read_write> readout_scores: array<atomic<u32>>;
@group(0) @binding(3) var<uniform>             params: GpuParams;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let group_size = 50u;
    let num_classes = 2u;
    let k = gid.x;
    if (k >= num_classes * group_size) { return; }

    let class_id = k / group_size;
    let node_idx = readout_groups[k];
    let act_fixed = u32(nodes[node_idx].activation * 1000000.0);
    atomicAdd(&readout_scores[class_id], act_fixed);
}
```

**Coût** : 100 threads → quasi-gratuit.

### C.9 `budget.wgsl` — Normalisation budgétaire (existant, conservé)

Aucun changement nécessaire. Les 2 passes (sum + scale) restent identiques.

---

## PARTIE D — Nouveau GpuContext

### D.1 Buffers à ajouter dans `GpuContext`

```rust
// NOUVEAUX buffers persistants
buf_nodes: wgpu::Buffer,           // GpuNode × num_nodes
buf_stimulus_groups: wgpu::Buffer, // u32 × (num_classes × group_size)
buf_zone_assignments: wgpu::Buffer,// u32 × num_nodes
buf_zone_state: wgpu::Buffer,      // GpuZone × num_zones
buf_zone_accum: wgpu::Buffer,      // atomic<u32> × num_zones
buf_csr_edge_indices: wgpu::Buffer,// u32 × num_edges (pour sync_conductances)
buf_readout_groups: wgpu::Buffer,  // u32 × (num_classes × group_size)
buf_readout_scores: wgpu::Buffer,  // atomic<u32> × num_classes
staging_readout: wgpu::Buffer,     // f32 × num_classes (MAP_READ)

// NOUVELLES pipelines
injection_pipeline: wgpu::ComputePipeline,
zone_measure_pipeline: wgpu::ComputePipeline,
zone_pid_pipeline: wgpu::ComputePipeline,
zone_apply_pipeline: wgpu::ComputePipeline,
source_contribs_pipeline: wgpu::ComputePipeline,
apply_dissipate_pipeline: wgpu::ComputePipeline,
sync_conductances_pipeline: wgpu::ComputePipeline,
readout_pipeline: wgpu::ComputePipeline,
```

### D.2 Méthode `run_full_tick()` — un seul submit

```rust
impl GpuContext {
    /// Exécuter un tick complet. Un seul CommandEncoder, un seul submit,
    /// zéro readback intermédiaire.
    pub fn run_full_tick(&self, params: &GpuParams, need_readout: bool) {
        // 1. Upload params
        self.queue.write_buffer(&self.buf_params, 0, bytemuck::bytes_of(params));

        // 2. Un seul encoder pour tout le tick
        let mut encoder = self.device.create_command_encoder(&Default::default());

        // Clear zone accumulators
        encoder.clear_buffer(&self.buf_zone_accum, 0, None);
        // Clear readout scores si besoin
        if need_readout {
            encoder.clear_buffer(&self.buf_readout_scores, 0, None);
        }

        let dispatch_nodes = (self.num_nodes + 255) / 256;
        let dispatch_edges = (self.num_edges + 255) / 256;

        // Phase 1: Injection stimulus
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.injection_pipeline);
            pass.set_bind_group(0, &self.injection_bg, &[]);
            pass.dispatch_workgroups(1, 1, 1); // ~50 threads
        }

        // Phase 2: Zone measure (accumulate activations)
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.zone_measure_pipeline);
            pass.set_bind_group(0, &self.zone_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes, 1, 1);
        }

        // Phase 3: Zone PID (1 thread per zone)
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.zone_pid_pipeline);
            pass.set_bind_group(0, &self.zone_bg, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }

        // Phase 4: Zone apply (theta_mod, gain_mod → nodes)
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.zone_apply_pipeline);
            pass.set_bind_group(0, &self.zone_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes, 1, 1);
        }

        // Phase 5: Source contributions
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.source_contribs_pipeline);
            pass.set_bind_group(0, &self.source_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes, 1, 1);
        }

        // Phase 6: Propagation (CSR)
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.propagation_pipeline);
            pass.set_bind_group(0, &self.propagation_bind_group, &[]);
            pass.dispatch_workgroups(dispatch_nodes, 1, 1);
        }

        // Phase 7: Apply influences + dissipation (fusionnés)
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.apply_dissipate_pipeline);
            pass.set_bind_group(0, &self.dissipate_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes, 1, 1);
        }

        // Phase 8: Plasticity (STDP + 3-factor + homeostasis + consolidation)
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.plasticity_pipeline);
            pass.set_bind_group(0, &self.plasticity_bind_group, &[]);
            pass.dispatch_workgroups(dispatch_edges, 1, 1);
        }

        // Clear budget_totals
        encoder.clear_buffer(&self.buf_budget_totals, 0, None);

        // Phase 9: Budget normalization (sum + scale)
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.budget_sum_pipeline);
            pass.set_bind_group(0, &self.budget_bind_group, &[]);
            pass.dispatch_workgroups(dispatch_edges, 1, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.budget_scale_pipeline);
            pass.set_bind_group(0, &self.budget_bind_group, &[]);
            pass.dispatch_workgroups(dispatch_edges, 1, 1);
        }

        // Phase 10: Sync conductances (edge → CSR order, GPU-to-GPU)
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.sync_conductances_pipeline);
            pass.set_bind_group(0, &self.sync_cond_bg, &[]);
            pass.dispatch_workgroups(dispatch_edges, 1, 1);
        }

        // Phase 11: Readout (seulement si nécessaire)
        if need_readout {
            {
                let mut pass = encoder.begin_compute_pass(&Default::default());
                pass.set_pipeline(&self.readout_pipeline);
                pass.set_bind_group(0, &self.readout_bg, &[]);
                pass.dispatch_workgroups(1, 1, 1); // ~100 threads
            }
            encoder.copy_buffer_to_buffer(
                &self.buf_readout_scores, 0,
                &self.staging_readout, 0,
                (self.num_classes as u64) * 4,
            );
        }

        // --- UN SEUL SUBMIT ---
        self.queue.submit(std::iter::once(encoder.finish()));

        // --- UN SEUL SYNC (seulement si readout nécessaire) ---
        if need_readout {
            self.device.poll(wgpu::Maintain::Wait);
        }
    }

    /// Lire les scores de readout après submit+poll.
    /// Retourne les scores par classe.
    pub fn read_readout_scores(&self, num_classes: usize) -> Vec<f64> {
        let slice = self.staging_readout.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| { tx.send(r).ok(); });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        let data = slice.get_mapped_range();
        let fixed: &[u32] = bytemuck::cast_slice(&data);
        let result: Vec<f64> = fixed.iter()
            .take(num_classes)
            .map(|&v| v as f64 / 1_000_000.0)
            .collect();
        drop(data);
        self.staging_readout.unmap();
        result
    }
}
```

### D.3 Key insight : `device.poll(Wait)` batché

Dans l'architecture actuelle :
```
submit → poll(Wait)     ← stall 2-3ms
submit → poll(Wait)     ← stall 2-3ms
submit → poll(Wait)     ← stall 2-3ms
                         = 6-9ms de stalls
```

Dans la nouvelle architecture :
```
submit (tout d'un coup)
// ...pas de readout → pas de poll ! Le GPU pipelinera seul.
// Le prochain tick fera queue.write_buffer qui synchronise implicitement.
```

**Pour les ticks sans readout (29 sur 31)** : zéro `poll(Wait)`, zéro stall.
Le GPU travaille pendant que le CPU prépare les params du tick suivant.

**Pour les ticks avec readout (1 sur 31)** : un seul `poll(Wait)`.

**Gain moyen** : ~7-9ms éliminés → passe de 21ms à ~3ms par tick.

---

## PARTIE E — Nouveau `Simulation::step()`

### E.1 Path GPU-First

```rust
impl Simulation {
    pub fn step(&mut self) -> TickMetrics {
        let tick = self.tick;
        let config = &self.config;

        self.perf.begin_tick();

        match &self.backend {
            ComputeBackend::Gpu(gpu) => {
                // === GPU-First path : tout tourne sur le GPU ===

                // Déterminer si ce tick nécessite un readout
                let need_readout = self.needs_readout(tick);

                // Déterminer le stimulus
                let stimulus_class = self.get_stimulus_class(tick);

                // Construire GpuParams (seul transfert CPU→GPU)
                let dopa_phasic = self.dopamine_system.level - config.dopamine.tonic;
                let gpu_params = GpuParams {
                    num_nodes: self.domain.num_nodes() as u32,
                    num_edges: self.domain.num_edges() as u32,
                    current_tick: tick as u32,
                    propagation_gain: config.propagation.gain as f32,
                    gain_inhibitory: config.propagation.gain_inhibitory as f32,
                    activation_decay: config.dissipation.activation_decay as f32,
                    activation_min: config.dissipation.activation_min as f32,
                    fatigue_gain: config.dissipation.fatigue_gain as f32,
                    fatigue_recovery: config.dissipation.fatigue_recovery as f32,
                    inhibition_gain: config.dissipation.inhibition_gain as f32,
                    inhibition_decay: config.dissipation.inhibition_decay as f32,
                    trace_gain: config.dissipation.trace_gain as f32,
                    trace_decay: config.dissipation.trace_decay as f32,
                    decay_jitter: config.dissipation.decay_jitter as f32,
                    stimulus_class: stimulus_class,
                    stimulus_intensity: config.input.intensity as f32,
                    // ... (STDP, dopamine, budget params) ...
                    global_delta_dopa: dopa_phasic as f32,
                    dopa_phasic: dopa_phasic as f32,
                    // ...
                };

                // === TOUT LE TICK EN UN SEUL APPEL ===
                gpu.run_full_tick(&gpu_params, need_readout);

                // Si readout nécessaire : lire les scores (seul readback, ~8 bytes)
                if need_readout {
                    let scores = gpu.read_readout_scores(config.output.num_classes);
                    self.process_readout(tick, &scores);
                }

                // Dopamine decay (CPU, trivial : 2 multiplications)
                self.dopamine_system.update(0.0, &config.dopamine);
            }

            ComputeBackend::Cpu => {
                // === Path CPU inchangé (fallback) ===
                // ... code existant identique ...
            }
        }

        self.perf.end_phase(Phase::GpuTick); // Nouveau : une seule phase GPU

        // Métriques légères (pas besoin de readback complet)
        let tick_metrics = self.lightweight_metrics(tick);

        self.perf.end_phase(Phase::Metrics);
        self.perf.end_tick(tick);
        self.tick += 1;

        tick_metrics
    }
}
```

### E.2 Métriques sans readback

Les métriques détaillées (activation moyenne, conductance, etc.) nécessitent
de lire l'état des nœuds/arêtes depuis le GPU. **On ne les calcule pas
à chaque tick.** Deux stratégies :

**Option A (recommandée)** : Shader `metrics.wgsl` qui fait un parallel
reduce sur le GPU et écrit quelques scalaires dans un petit buffer.

**Option B** : Readback complet de `buf_nodes` tous les N ticks
(ex: snapshot_interval = 100). Le coût est alors 50k × 48B = 2.4 MB,
amortis sur 100 ticks → ~24 KB/tick en moyenne.

Pour le profiling, on recommande l'option A :

```wgsl
// metrics_reduce.wgsl — parallel reduce pour métriques agrégées
// Produit : active_count, sum_energy, max_activation, mean_conductance
// en un seul dispatch avec workgroup shared memory reduction
```

### E.3 Synchronisation de l'état CPU (mode viz)

En mode visualisation, le CPU doit connaître l'état des nœuds pour le rendu.
On ajoute une méthode `download_nodes_snapshot()` qui copie `buf_nodes`
vers le CPU. Cette méthode n'est appelée que par la viz, à la fréquence de
rendu (tous les 16ms), pas à chaque tick.

```rust
impl GpuContext {
    pub fn download_nodes_snapshot(&self, nodes: &mut [GpuNodeCpu]) {
        // ... copy buf_nodes → staging_nodes → map_async → copie ...
    }
}
```

---

## PARTIE F — Optimisations de la visualisation

### F.1 Principes retenus

| ID | Technique | Retenu | Impact |
|----|-----------|--------|--------|
| 2 | Culling : ne dessiner que les nœuds actifs | **OUI** | ~10× moins de draw calls |
| 3 | `ticks_per_frame = 1` par défaut | **OUI** | Frame budget = 1 sim tick |
| 5 | Thread séparé pour la simulation | **OUI** | Décorrèle sim et rendu → 60 FPS |

### F.2 Architecture thread séparé (F5)

```
┌─── THREAD SIMULATION ──────────┐     ┌─── THREAD RENDU (main) ─────────┐
│                                 │     │                                  │
│  loop {                         │     │  loop {                          │
│    sim.step();  // ~3ms/tick    │     │    // Lire le dernier snapshot   │
│    // publish snapshot          │     │    let snap = shared.lock();     │
│    shared.lock() = snapshot;    │     │    draw_points_3d(&snap);       │
│    tick += 1;                   │     │    draw_sidebar(&snap);         │
│  }                              │     │    next_frame().await;  //16ms  │
│                                 │     │                                  │
└─────────────────────────────────┘     └──────────────────────────────────┘
          ▲                                         │
          │         Arc<Mutex<VizSnapshot>>          │
          └─────────────────────────────────────────┘
```

```rust
/// Snapshot léger pour la viz — contient uniquement les données de rendu.
struct VizSnapshot {
    tick: usize,
    /// Position 3D de chaque nœud (constante, partagée par Arc)
    positions: Arc<Vec<[f64; 3]>>,
    /// Activation de chaque nœud (mis à jour depuis GPU)
    activations: Vec<f32>,
    /// Type E/I par nœud (constant)
    is_excitatory: Arc<Vec<bool>>,
    /// Liste des nœuds actifs (pour culling)
    active_indices: Vec<usize>,
    /// Métriques agrégées (sidebar)
    metrics: VizMetrics,
}

struct VizMetrics {
    active_count: usize,
    total_energy: f64,
    max_activation: f64,
    mean_conductance: f64,
    max_conductance: f64,
    mean_trace: f64,
    max_trace: f64,
    mean_fatigue: f64,
    dopamine_level: f64,
    current_trial: usize,
    total_trials: usize,
    correct_count: usize,
    total_evaluated: usize,
}
```

### F.3 Culling des nœuds inactifs (F2)

Dans `draw_points_3d`, au lieu d'itérer sur les 50k nœuds :

```rust
// AVANT (50k draw calls) :
for (i, &(px, py)) in projected.iter().enumerate() {
    let normalized = (values[i] / max_val) as f32;
    draw_rectangle(...);
}

// APRÈS (culling — ~2500-5000 draw calls) :
// Option A : ne dessiner que les nœuds actifs + fond statique
for &i in &snapshot.active_indices {
    let (px, py) = projected[i];
    let normalized = (snapshot.activations[i] / max_val) as f32;
    draw_rectangle(..., heat_color(normalized));
}

// Option B : dessiner tous les nœuds mais avec early-skip
for (i, &(px, py)) in projected.iter().enumerate() {
    let act = snapshot.activations[i];
    // Skip les nœuds au repos (activation < threshold)
    if act < 0.01 {
        continue; // La couleur de fond suffit
    }
    draw_rectangle(...);
}
```

L'option B est plus simple et plus visuelle (on voit quand même le fond).
On dessine le fond une seule fois (couleur inhibitory/excitatory dim) lors du
premier frame ou quand la rotation change, puis on ne dessine que les nœuds
actifs en overlay.

### F.4 `ticks_per_frame = 1` par défaut (F3)

```rust
// Dans VizState::new() :
VizState {
    ticks_per_frame: 1,  // était 4
    // ...
}
```

Avec la nouvelle architecture GPU-first (tick ~3ms), on peut monter à
`ticks_per_frame = 4` tout en restant dans le budget de 16ms.

### F.5 Rendu fond statique + overlay actif

Pour éviter de re-dessiner 50k rectangles à chaque frame :

1. À l'init et au changement de rotation, dessiner tous les nœuds
   en couleur sombre dans une `RenderTarget` / `Image` off-screen.
2. À chaque frame, copier cette image puis dessiner **uniquement** les
   nœuds actifs par-dessus.

Cela n'est PAS du "texture rendering" complet (on garde `draw_rectangle` pour
les nœuds actifs), mais ça réduit le coût de base à ~0 draw calls pour le
fond.

---

## PARTIE G — Optimisations additionnelles

### G.1 Précision f32 uniforme

**Problème** : L'architecture actuelle utilise f64 côté CPU et f32 côté GPU.
Chaque transfert nécessite une conversion. L'état `Domain` stocke les nœuds
en f64 (cf. `Node { activation: f64, ... }`), les edges en f64, les
conductances en f64.

**Solution** : Passer les structures CPU en f32 pour l'état chaud.
La précision f64 n'est pas requise pour des activations, conductances,
seuils — les frameworks de deep learning (PyTorch, JAX, etc.) utilisent
f32 voire f16 pour l'entraînement.

**Impact** :
- Élimine toutes les conversions f64↔f32 dans les uploads/downloads
- Réduit la taille de `Node` de ~120 bytes à ~48 bytes → meilleur cache L1/L2
- Réduit la taille de `Edge` de ~80 bytes à ~32 bytes
- Le path CPU (rayon) bénéficie aussi de la réduction de footprint mémoire

**Fichiers impactés** : `node.rs`, `edge.rs`, `domain.rs`, `propagation.rs`,
`stdp.rs`, `zone.rs`, `metrics.rs`, `simulation.rs`.

### G.2 Double buffering asynchrone (optionnel, avancé)

Pour les ticks sans readout (29/31), le CPU n'attend jamais la fin du GPU.
On peut aller plus loin avec du double buffering :

```
Tick T   : CPU prépare params T → submit T
Tick T+1 : CPU prépare params T+1 → submit T+1
           (Le GPU exécute T pendant que le CPU prépare T+1)
```

wgpu assure automatiquement l'ordonnancement des commandes sur la même queue.
Le CPU ne fait `poll(Wait)` que quand il a besoin d'un readback.

### G.3 Éviter `encoder.clear_buffer` pour zone_accum

Au lieu de clear le buffer zone_accum à chaque tick, le shader zone_measure
peut écrire `0` au début si `gid.x < num_zones` (vu que zone_measure dispatch
num_nodes threads, les premiers threads initialisent).

Plus proprement : utiliser un dispatch dédié minuscule (1 workgroup) pour
le clear, ou alterner entre deux buffers (ping-pong).

### G.4 Profiling GPU avec timestamp queries

Ajouter des `timestamp_writes` aux compute passes pour mesurer le temps
GPU réel de chaque phase, sans les stalls CPU :

```rust
let ts_set = device.create_query_set(&wgpu::QuerySetDescriptor {
    ty: wgpu::QueryType::Timestamp,
    count: 24, // 2 par phase (début + fin)
    label: Some("perf_timestamps"),
});
```

Cela permettra de savoir exactement combien de temps chaque shader prend
sans pollution par les stalls.

### G.5 Workgroup size adaptatif

Les shaders utilisent tous `workgroup_size(256)`. Sur certains GPU (surtout
intégrés), 128 ou 64 peut être plus efficace. Rendre cela paramétrable
via `ComputeConfig::gpu_workgroup_size` (déjà dans la config mais pas
utilisé dynamiquement).

### G.6 Shared memory pour propagation CSR

Le shader `propagation.wgsl` fait un inner loop sur les arêtes CSR entrantes.
Pour les nœuds avec beaucoup de voisins, utiliser de la shared memory
pour pré-charger les `source_contribs` des nœuds voisins réduirait les
accès mémoire globale.

### G.7 Métriques GPU-side (shader reduce)

Ajouter un shader `metrics_reduce.wgsl` qui calcule les métriques agrégées
(active_count, sum_energy, max_activation, etc.) directement sur le GPU
via une réduction en shared memory, évitant le readback de l'état complet.

Output : petit buffer ~64 bytes contenant les scalaires agrégés.

---

## PARTIE H — Checklist d'implémentation

### Phase 1 : Fondations GPU-First (critique)

- [ ] **H1**. Définir `GpuNode` en `bytemuck` Rust + WGSL, ajouter `buf_nodes` dans `GpuContext`
- [ ] **H2**. Créer `injection.wgsl` + pipeline + bind group
- [ ] **H3**. Créer `zones.wgsl` (3 entry points) + buffer zone_state + zone_assignments + pipelines
- [ ] **H4**. Créer `source_contribs.wgsl` + pipeline — remplace `compute_source_contribs()` CPU
- [ ] **H5**. Créer `apply_and_dissipate.wgsl` + pipeline — remplace `apply_influences()` + dissipation CPU
- [ ] **H6**. Créer `sync_conductances.wgsl` + buffer `csr_edge_indices` GPU — élimine le round-trip
- [ ] **H7**. Modifier `plasticity.wgsl` pour lire `last_activation_tick` depuis `buf_nodes`
- [ ] **H8**. Créer `readout.wgsl` + `buf_readout_scores` + `buf_readout_groups`
- [ ] **H9**. Implémenter `GpuContext::run_full_tick()` — un seul encoder, un seul submit
- [ ] **H10**. Réécrire `Simulation::step()` path GPU pour utiliser `run_full_tick()`
- [ ] **H11**. Uploader l'état initial des nœuds dans `buf_nodes` au setup

### Phase 2 : Optimisations complémentaires

- [ ] **H12**. Passer `Node`/`Edge`/`Domain` en f32 (G.1)
- [ ] **H13**. Ajouter shader `metrics_reduce.wgsl` pour métriques GPU-side (G.7)
- [ ] **H14**. Implémenter timestamp queries pour profiling GPU (G.4)

### Phase 3 : Visualisation

- [ ] **H15**. Extraire `VizSnapshot` et `VizMetrics` — préparer le threading
- [ ] **H16**. Thread séparé pour la simulation (`std::thread::spawn` + `Arc<Mutex<VizSnapshot>>`)
- [ ] **H17**. Culling des nœuds inactifs dans `draw_points_3d`
- [ ] **H18**. Défaut `ticks_per_frame = 1`
- [ ] **H19**. `download_nodes_snapshot()` pour la viz (readback partiel à la fréquence rendu)

### Phase 4 : Polish

- [ ] **H20**. Conservation du path CPU (fallback) — s'assurer qu'il compile encore
- [ ] **H21**. Tests de validation : comparer CPU vs GPU sur 1000 ticks, vérifier convergence
- [ ] **H22**. Benchmarks comparatifs avant/après

---

## Résumé des gains attendus

| Métrique | Avant (V4.2) | Après (V4.3) | Facteur |
|----------|-------------|-------------|---------|
| Stalls GPU/tick | 3 × 2-3ms = 7-9ms | 0ms (29/31 ticks) | **∞** |
| Transfert GPU↔CPU/tick | 20.8 MB | ~150 B | **140 000×** |
| Conversions f64↔f32/tick | 2 × 577k + 2 × 50k | 0 | **∞** |
| Temps PLAST+STDP | 21ms | ~3ms (compute pur) | **7×** |
| Temps total/tick | 29ms | **~3-5ms** | **6-10×** |
| Ticks/sec | 34 | **200-330** | **6-10×** |
| Viz FPS | ~7 FPS | **60 FPS** (threaded) | **8×** |
| Readback/tick (CLI) | 18.5 MB | 0 B (29/31 ticks) | **∞** |