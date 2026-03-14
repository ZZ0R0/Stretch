# V4.2 — Plan d'optimisation GPU + Framework pérenne

> **Baselines perf mesurées :**
> - Config 50k nœuds, 577k arêtes, 200 ticks : **10.81 ms/tick**
> - Config réelle ticks 500-600 : **114.19 ms/tick** (zones:12ms, PROPAG:11.7ms, **PLAST+STDP:83.7ms**, dissip:4.5ms, metrics:2.1ms)
>
> **Objectif :** Descendre sous **5 ms/tick** pour 50k nœuds via GPU compute + optimisations algorithmiques.
> Construire un framework modulaire, pérenne et réutilisable.
>
> **Machine de dev :** AMD Barcelo (iGPU) — pas de NVIDIA. Le code GPU doit fonctionner via **wgpu** (Vulkan/Metal/DX12) pour le dev, et supporter NVIDIA nativement en production.

---

## A. CORRECTIONS PRÉCÉDENTES — Bilan (toutes ✅)

| # | Correction | Fichiers modifiés | Statut |
|---|-----------|-------------------|--------|
| A1 | Suppression `enabled: bool` (9 sous-systèmes) | config.rs, dopamine.rs, reward.rs, input.rs, output.rs | ✅ |
| A2 | Suppression `plasticity.rs` (code mort) | lib.rs, plasticity.rs supprimé | ✅ |
| A3 | Pipeline plasticité unifié (STDP→élig→3-facteurs→homéo→conso→budget) | stdp.rs | ✅ |
| A4 | Consolidation gatée dopamine | stdp.rs | ✅ |
| A5 | `edge_defaults.decay` intégré au pipeline | stdp.rs | ✅ |
| A7 | InputEncoder remplace stimuli classiques | simulation.rs | ✅ |
| A8 | CLI utilise `setup_v4_training()` | cli/main.rs | ✅ |
| A10 | Ancien `compute_influences()` supprimé | propagation.rs | ✅ |
| A11 | Métriques V4 complètes | metrics.rs | ✅ |
| B1 | KD-tree persisté dans Domain | domain.rs | ✅ |
| B2 | kernel_weight pré-calculés dans CSR | domain.rs | ✅ |
| B4 | CSR (IncomingCSR) pour arêtes entrantes | domain.rs | ✅ |
| B5 | Fired list (propagation sur ~5% actifs) | propagation.rs | ✅ via source_contribs skip |
| B7 | Bitmap `node_is_excitatory` | domain.rs, propagation.rs | ✅ |

**Restant du plan A/B non encore implémenté :**

| # | Correction | Statut |
|---|-----------|--------|
| B8 | Arêtes éligibles seulement (hot edges list) | 🔲 → C3 ci-dessous |

---

## B. ANALYSE PERFORMANCE — Profil actuel

### B.1 Répartition du temps par phase (ticks 500-600, config 50k nœuds)

| Phase | min (ms) | moy (ms) | max (ms) | % total |
|-------|---------|---------|---------|---------|
| zones | 7.32 | 11.98 | 21.71 | 10.5% |
| stim+input | 0.00 | 0.15 | 5.31 | 0.1% |
| **PROPAG** | 8.75 | 11.74 | 19.03 | **10.3%** |
| dissip | 3.51 | 4.50 | 8.56 | 3.9% |
| **PLAST+STDP** | 71.95 | **83.73** | 133.68 | **73.3%** |
| readout+rew | 0.00 | 0.00 | 0.01 | 0.0% |
| metrics | 0.00 | 2.09 | 22.28 | 1.8% |
| **TOTAL** | | **114.19** | | **100%** |

### B.2 Analyse du goulot PLAST+STDP (83.73 ms, 73% du temps)

Le pipeline dans `stdp.rs` itère **toutes les 577k+ arêtes** à chaque tick :

```
Phase A : last_activation_tick (50k nœuds, par_iter_mut)         ~0.2ms
Phase B : snapshot activation_ticks (50k, séquentiel)            ~0.1ms
Phase C : boucle sur 577k arêtes × 5 opérations/arête :         ~75ms
  1. STDP ψ : 2 lookups activation_ticks + exp()                
  2. Eligibility : multiply-add + clamp                          
  3. Trois-facteurs : dopa × eligibility → conductance           
  4. Homéostasie : decay vers baseline                           
  5. Consolidation : condition dopamine + update counter          
Phase D : budget synaptique (2 passes sur 577k arêtes)           ~8ms
```

**Problème clé :** ~95% des arêtes ont ψ=0 (ni pré ni post neurone actif récemment) ET eligibility≈0 (décroissance rapide). Pourtant elles sont toutes traitées.

### B.3 Analyse PROPAG (11.74 ms, 10.3%)

```
compute_source_contribs : 50k nœuds, ~5% actifs              ~2ms
compute_influences_csr  : 577k arêtes CSR, dot-product        ~8ms
apply_influences        : 50k nœuds, clamp                    ~1.5ms
```

Le CSR + kernel_weights pré-calculés sont déjà en place. Le bottleneck restant est le dot-product sur toutes les arêtes entrantes, même quand `source_contribs[src] == 0`.

### B.4 Analyse Zones (11.98 ms, 10.5%)

8 zones PID, chaque zone itère ses ~6250 membres pour mesurer l'activité. Le PID indirect modifie `threshold_mod` et `gain_mods` par nœud. Overhead lié au pattern Zone→membres (scatter).

---

## C. OPTIMISATIONS ALGORITHMIQUES (CPU pur, Rust)

### C1. Fired-only propagation ⬚
**Fichier :** `propagation.rs` → `compute_influences_csr()`
**Problème :** Le CSR itère toutes les arêtes entrantes pour chaque nœud cible, même si `source_contribs[src] == 0` pour ~95% des sources.
**Solution :** Maintenir une `fired_list: Vec<usize>` (nœuds avec source_contrib ≠ 0). Construire un CSR **sortant** (outgoing). Itérer uniquement les nœuds actifs et accumuler dans un buffer influences atomique ou par chunks.
**Impact estimé :** PROPAG 11.7ms → ~2-3ms
**État actuel du code :**
- `propagation.rs:compute_source_contribs()` calcule déjà source_contribs avec skip des inactifs
- `propagation.rs:compute_influences_csr()` utilise `domain.incoming` (IncomingCSR) — target-centric, pas source-centric
- Il faudrait ajouter un `OutgoingCSR` dans `domain.rs` (ou réutiliser `adjacency: Vec<Vec<usize>>`) et changer l'approche

**Implémentation détaillée :**
1. Dans `domain.rs` : ajouter `pub outgoing_csr: OutgoingCSR` (même structure que IncomingCSR mais indexé par source)
2. Dans `propagation.rs` : nouvelle fn `compute_influences_fired_only()` qui :
   - Construit `fired_list` depuis `buf_source_contribs` (indices non-nuls)
   - Pour chaque fired node, itère ses arêtes sortantes via outgoing_csr
   - Accumule `source_contrib * kernel_weight * conductance` dans `influences_buf[target]` via `AtomicF64` ou chunk-local buffers
3. Fallback: si fired_ratio > 30%, utiliser l'ancien CSR entrant (plus efficace quand beaucoup de nœuds actifs)

### C2. Skip STDP ψ pour arêtes inactives ⬚
**Fichier :** `stdp.rs` → Phase C, étape 1
**Problème :** Le calcul de ψ (STDP) requiert `exp()` mais 95% des arêtes ont `activation_ticks[from] == None` ou `activation_ticks[to] == None` → ψ=0 systématiquement.
**Solution :** Plutôt que vérifier `if let (Some, Some)` sur chaque arête (coûteux car 577k lookups), maintenir un bitset `recently_active[node_idx]` et ne calculer ψ que pour les arêtes dont **au moins un** des deux nœuds a tiré dans les derniers `tau_plus` ticks.
**Impact estimé :** Réduit les calculs ψ de 577k à ~50k arêtes (~10x)
**État actuel du code :**
- `stdp.rs:Phase C` fait `activation_ticks[edge.from]` + `activation_ticks[edge.to]` pour chaque arête
- Le `if let (Some(t_pre), Some(t_post))` court-circuite déjà, mais on paie quand-même le coût du parcours de 577k arêtes + 2 lookups par arête
- La solution optimale est de ne parcourir que les arêtes touchant un nœud récemment actif

**Implémentation détaillée :**
1. Maintenir `active_edge_indices: Vec<usize>` = indices des arêtes où `from` ou `to` a tiré dans les derniers `max(tau_plus, tau_minus)` ticks
2. Construire cette liste en utilisant les adjacency sortantes+entrantes des nœuds ayant tiré
3. Phase C étape 1 (STDP ψ) : itérer `active_edge_indices` seulement

### C3. Hot edges list pour trois-facteurs ⬚
**Fichier :** `stdp.rs` → Phase C, étapes 2-5
**Problème :** Les étapes 2 (eligibility update), 3 (trois-facteurs), 4 (homéostasie), 5 (consolidation) touchent **toutes les 577k arêtes** alors que les étapes 3-5 n'ont d'effet significatif que sur les arêtes avec `|eligibility| > ε`.
**Solution :** Maintenir une `hot_edges: Vec<usize>` — liste des indices d'arêtes dont `|eligibility| > ε` (ε = 1e-6). Mise à jour incrémentale :
- Quand ψ ≠ 0 et que l'arête n'est pas dans hot_edges → ajouter
- Quand eligibility tombe sous ε → retirer (lazy : via compaction périodique)
**Impact estimé :** PLAST+STDP 83ms → ~8-15ms (on ne traite que ~5-10% des arêtes)
**État actuel du code :**
- `stdp.rs:Phase C` fait `par_iter_mut().for_each()` sur `domain.edges` (toutes)
- L'éligibilité décroît avec `decay=0.85` → demi-vie ~4 ticks → la plupart des arêtes sont à ~0 rapidement
- Les étapes 3-5 sont wasteful si `eligibility ≈ 0` : `dw = η × δ_d × 0 = 0`, homéostasie negligéable
**Implémentation détaillée :**
1. Ajouter dans `Domain` : `pub hot_edge_indices: Vec<usize>` + `pub edge_is_hot: Vec<bool>` (bitmap)
2. Après Phase C étapes 1-2 : scanner les arêtes de `active_edge_indices` (C2) et marquer celles avec `|eligibility| > ε` comme hot
3. Étapes 3-5 : itérer `hot_edge_indices` uniquement
4. Phase D (budget) : ne recalculer que les nœuds sources ayant au moins une hot edge modifiée
5. Compaction : tous les N ticks, retirer les arêtes avec `|eligibility| < ε` de hot_edges

### C4. Budget synaptique incrémental ⬚
**Fichier :** `stdp.rs` → Phase D
**Problème :** La normalisation fait 2 passes complètes sur 577k arêtes :
  - Pass 1 : sum conductances par source (`adjacency.par_iter()` → 577k)
  - Pass 2 : scale down si > budget (`edges.par_iter_mut()` → 577k)
**Solution :** Ne recalculer les totaux que pour les nœuds source dont au moins une arête a été modifiée (via `dirty_sources: HashSet<usize>` ou bitmap). En pratique, seuls ~5-10% des nœuds source ont des hot edges.
**Impact estimé :** Phase D 8ms → ~1ms
**État actuel du code :**
- `totals: Vec<f64>` est recalculé entièrement chaque tick
- `domain.adjacency` (Vec<Vec<usize>>) est utilisé pour la somme → cache-unfriendly
**Implémentation détaillée :**
1. Maintenir `running_totals: Vec<f64>` (par nœud source) dans Domain, initialisé au setup
2. Lors de C3, tracker `old_conductance` et `new_conductance` pour chaque hot edge → `delta = new - old`
3. `running_totals[edge.from] += delta`
4. Phase D : ne scale que les nœuds dont `running_totals[i] > budget`

### C5. Dissipation : skip nœuds à l'équilibre ⬚
**Fichier :** `simulation.rs` → Phase 5 (dissipation)
**Problème :** 50k nœuds × 6 opérations (fatigue, inhibition, trace, excitability, decay, clamp). Les nœuds au repos (`activation ≈ activation_min`, `fatigue ≈ 0`, `inhibition ≈ 0`) n'ont pas besoin de mise à jour.
**Solution :** Bitmap `node_needs_update[i]` = nœud a reçu une influence > 0 ou a tiré récemment. Ne dissiper que ces nœuds.
**Impact estimé :** dissip 4.5ms → ~1ms
**Dépendance :** Requiert de maintenir le bitmap via la propagation (marquer les nœuds dont `influences[i] > threshold`)

### C6. Zones : skip zones stables ⬚
**Fichier :** `zone.rs` → `measure()` + `regulate()`
**Problème :** 8 zones × ~6250 membres. La mesure itère tous les membres même si l'erreur PID est quasi-nulle.
**Solution :** Si `|error| < ε` pendant N ticks consécutifs → skip la zone (timer de réactivation si erreur remonte).
**Impact estimé :** zones 12ms → ~3ms (la plupart des zones convergent après ~100 ticks)

---

## D. GPU COMPUTE — Architecture et interfaçage

### D0. Choix technique : `wgpu` (Vulkan/Metal/DX12) ⬚

**Pourquoi `wgpu` plutôt que CUDA/cudarc :**
- La machine de dev a un AMD Barcelo (iGPU Vulkan) — pas de NVIDIA
- `wgpu` fonctionne sur **tous les GPU** : AMD (Vulkan), NVIDIA (Vulkan), Apple (Metal), Intel
- En production avec NVIDIA, wgpu utilise le driver Vulkan NVIDIA (performances quasi-identiques à CUDA pour les compute shaders)
- Crate Rust native, pas de toolchain CUDA à installer
- Si besoin de CUDA spécifique plus tard (tensor cores, etc.), on peut ajouter `cudarc` comme backend alternatif

**Dépendances Cargo :**
```toml
# stretch-core/Cargo.toml
[dependencies]
wgpu = "24"        # GPU compute shaders
bytemuck = { version = "1", features = ["derive"] }  # Cast structs → GPU buffers
pollster = "0.4"   # Block on async wgpu init
```

### D1. Module `gpu.rs` — Abstraction GPU ⬚
**Nouveau fichier :** `stretch-core/src/gpu.rs`
**Rôle :** Encapsuler l'initialisation wgpu, la gestion des buffers GPU, et l'exécution des compute shaders.

**Structure prévue :**
```rust
pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    // Pipelines pré-compilés
    plasticity_pipeline: wgpu::ComputePipeline,
    propagation_pipeline: wgpu::ComputePipeline,
    budget_pipeline: wgpu::ComputePipeline,
    // Buffers GPU persistants (alloués une fois au setup)
    buf_edges: wgpu::Buffer,          // [from, to, conductance, eligibility, consolidated, ...]
    buf_activation_ticks: wgpu::Buffer, // [Option<usize>; n_nodes]
    buf_source_contribs: wgpu::Buffer,  // [f64; n_nodes]
    buf_conductances: wgpu::Buffer,     // [f64; n_edges]
    buf_csr_offsets: wgpu::Buffer,      // CSR entrant
    buf_csr_sources: wgpu::Buffer,
    buf_csr_kernels: wgpu::Buffer,
    buf_influences: wgpu::Buffer,       // [f64; n_nodes]
    buf_node_delta_dopa: wgpu::Buffer,  // [f64; n_nodes]
    buf_params: wgpu::Buffer,           // Struct uniforme avec tous les paramètres scalaires
    // Bind groups
    plasticity_bind_group: wgpu::BindGroup,
    propagation_bind_group: wgpu::BindGroup,
}
```

**API publique :**
```rust
impl GpuContext {
    pub fn new(domain: &Domain, config: &SimConfig) -> Self;
    pub fn upload_activation_ticks(&self, ticks: &[Option<usize>]);
    pub fn upload_source_contribs(&self, contribs: &[f64]);
    pub fn upload_params(&self, params: &GpuParams);
    pub fn run_plasticity(&self) -> Vec<f64>;      // → conductances mises à jour
    pub fn run_propagation(&self) -> Vec<f64>;     // → influences
    pub fn run_budget_norm(&self);
    pub fn download_conductances(&self) -> Vec<f64>;
    pub fn download_influences(&self) -> Vec<f64>;
}
```

**Cycle par tick :**
1. CPU calcule source_contribs (léger, 50k nœuds) + snapshot activation_ticks
2. Upload source_contribs + activation_ticks → GPU
3. GPU exécute propagation shader → influences
4. Download influences → CPU
5. CPU applique influences + dissipation
6. GPU exécute plasticity shader (STDP+eligibility+3-facteurs+homéo+conso)
7. GPU exécute budget normalization shader
8. Download conductances → CPU (sync_conductances)

### D2. Shader PLAST+STDP (`plasticity.wgsl`) ⬚
**Nouveau fichier :** `stretch-core/shaders/plasticity.wgsl`
**Une invocation par arête** (577k workgroups de 256 threads = ~2256 dispatches).

```wgsl
// Pseudo-WGSL — chaque thread traite une arête
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let edge_idx = gid.x;
    if (edge_idx >= params.num_edges) { return; }
    
    let from = edges[edge_idx].from;
    let to = edges[edge_idx].to;
    
    // 1. STDP ψ
    let t_pre = activation_ticks[from];
    let t_post = activation_ticks[to];
    var psi: f32 = 0.0;
    if (t_pre >= 0 && t_post >= 0 && t_pre != t_post) {
        let dt = f32(t_post - t_pre);
        if (dt > 0.0) { psi = params.a_plus * exp(-dt / params.tau_plus); }
        else { psi = -params.a_minus * exp(dt / params.tau_minus); }
    }
    
    // 2. Eligibility
    edges[edge_idx].eligibility = clamp(
        params.elig_decay * edges[edge_idx].eligibility + psi,
        -params.elig_max, params.elig_max
    );
    
    // 3. Trois-facteurs
    let delta_d = select(params.global_delta_dopa, 
                         params.dopa_phasic * node_delta_dopa[to],
                         params.use_spatial);
    let dw = params.plasticity_gain * delta_d * edges[edge_idx].eligibility;
    edges[edge_idx].conductance = clamp(
        edges[edge_idx].conductance + dw,
        params.cond_min, params.cond_max
    );
    
    // 4. Homéostasie
    if (!edges[edge_idx].consolidated) {
        edges[edge_idx].conductance += params.homeostatic_rate 
            * (params.baseline_cond - edges[edge_idx].conductance);
        edges[edge_idx].conductance = clamp(edges[edge_idx].conductance, 
            params.cond_min, params.cond_max);
    }
    
    // 5. Consolidation
    if (params.dopamine_level > params.dopa_consol_threshold 
        && edges[edge_idx].eligibility > 0.0) {
        // consolidation counter update
    }
}
```

**Performance estimée :** 577k arêtes × ~20 FP ops / arête = ~11.5M FLOPs. Un GPU mid-range (AMD Barcelo ~1 TFLOPS) : **~0.01ms**. En pratique avec overhead mémoire : **~0.5-2ms** (vs 83ms CPU).

### D3. Shader PROPAG (`propagation.wgsl`) ⬚
**Nouveau fichier :** `stretch-core/shaders/propagation.wgsl`
**Une invocation par nœud cible** (50k threads). Chaque thread itère ses arêtes entrantes via CSR.

```wgsl
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let node_idx = gid.x;
    if (node_idx >= params.num_nodes) { return; }
    
    let start = csr_offsets[node_idx];
    let end = csr_offsets[node_idx + 1];
    var total: f32 = 0.0;
    for (var k = start; k < end; k++) {
        let src = source_contribs[csr_sources[k]];
        if (src == 0.0) { continue; }
        total += src * conductances[csr_edges[k]] * csr_kernels[k];
    }
    influences[node_idx] = total;
}
```

**Performance estimée :** 50k nœuds × ~12 arêtes/nœud = 600k multiply-adds. **~0.1-0.5ms** (vs 11.7ms CPU).

### D4. Shader Budget (`budget.wgsl`) ⬚
**Nouveau fichier :** `stretch-core/shaders/budget.wgsl`
**Deux dispatches :**
1. Pass 1 : somme des conductances par source (parallel reduction avec atomics)
2. Pass 2 : scale down si > budget

### D5. Fallback CPU transparent ⬚
**Fichier :** `simulation.rs`
**Si le GPU n'est pas disponible** (ex: CI, serveur sans GPU), le code doit fonctionner en mode CPU pur comme aujourd'hui. Le choix CPU/GPU est fait au `Simulation::new()` via un `enum ComputeBackend { Cpu, Gpu(GpuContext) }`.

```rust
pub enum ComputeBackend {
    Cpu,
    Gpu(GpuContext),
}
```

Au `step()`, chaque phase dispatch vers le backend approprié :
```rust
match &self.backend {
    ComputeBackend::Cpu => { /* code actuel */ }
    ComputeBackend::Gpu(gpu) => { gpu.run_plasticity(); }
}
```

---

## E. FRAMEWORK — Architecture modulaire et pérenne

### E1. Réorganisation en crates ⬚

Structure cible :
```
stretch-core/          # Structures de données + logique simulation
  src/
    domain.rs          # Domain, IncomingCSR, OutgoingCSR — le graphe spatial
    node.rs            # Node, NeuronType
    edge.rs            # Edge
    config.rs          # SimConfig et sous-configs
    simulation.rs      # Simulation orchestrator (CPU dispatch)
    propagation.rs     # Propagation (CPU impl)
    stdp.rs            # Plasticité (CPU impl)
    zone.rs            # Zones PID
    dopamine.rs        # Système dopaminergique
    reward.rs          # Récompense
    input.rs           # InputEncoder
    output.rs          # OutputReader
    pacemaker.rs       # Pacemakers
    stimulus.rs        # Stimuli
    metrics.rs         # Métriques
    perf.rs            # PerfMonitor
    gpu.rs             # ← NOUVEAU : GpuContext + compute shaders
    lib.rs
  shaders/             # ← NOUVEAU : répertoire des compute shaders WGSL
    plasticity.wgsl
    propagation.wgsl
    budget.wgsl
stretch-cli/           # CLI batch runner
stretch-viz/           # Visualisation temps réel (macroquad)
```

**Pas de sur-découpage** : stretch-core reste un seul crate pour éviter la complexité de compilation inter-crates. Les modules internes sont bien séparés mais ne nécessitent pas de crates séparés pour l'instant.

### E2. Briques réutilisables identifiées ⬚

| Brique | Module | Réutilisabilité |
|--------|--------|-----------------|
| **IncomingCSR / OutgoingCSR** | `domain.rs` | Tout graphe sparse — réutilisable pour GNN, transport, etc. |
| **GpuContext** | `gpu.rs` | Tout compute shader wgpu — réutilisable pour d'autres simulations |
| **PerfMonitor** | `perf.rs` | Tout pipeline multi-phases — métriques min/moy/max |
| **Three-factor learning** | `stdp.rs` | Tout réseau de neurones à apprentissage par récompense |
| **ZoneManager + PID** | `zone.rs` | Tout système nécessitant un contrôle PID spatial |
| **KD-tree spatial** | `domain.rs` | Tout graphe 3D avec requêtes de proximité |
| **Trial scheduler** | `simulation.rs` | Tout protocole expérimental avec stimulation séquentielle |

### E3. Traits d'abstraction ⬚
**Fichier :** `simulation.rs` (ou nouveau `traits.rs`)

Pour rendre les briques interchangeables :

```rust
/// Backend de calcul (CPU ou GPU)
pub trait ComputeEngine {
    fn compute_influences(&self, domain: &Domain, source_contribs: &[f64], out: &mut [f64]);
    fn update_plasticity(&self, domain: &mut Domain, params: &PlasticityParams);
    fn normalize_budget(&self, domain: &mut Domain, budget: f64);
}

/// Règle de plasticité (STDP, Hebbian, ou future)
pub trait PlasticityRule {
    fn update(&self, domain: &mut Domain, tick: usize, params: &dyn std::any::Any);
}

/// Protocole expérimental
pub trait ExperimentProtocol {
    fn next_trial(&mut self, tick: usize) -> Option<Trial>;
    fn evaluate(&mut self, decision: usize, target: usize) -> f64; // reward
}
```

### E4. Configuration par environnement ⬚
**Fichier :** `config.rs`

Ajouter une section `[compute]` dans le TOML :
```toml
[compute]
backend = "auto"       # "auto" | "cpu" | "gpu"
gpu_workgroup_size = 256
gpu_device_index = 0   # si plusieurs GPU
```

Le `"auto"` tente le GPU, fallback CPU si échec.

---

## F. DETTE TECHNIQUE RÉSIDUELLE

### F1. Supprimer `enabled` résiduel dans config structs ⬚
**Fichiers :** `config.rs` (ZoneConfig, ConsolidationConfig, NeuronTypesConfig, StdpConfig, SynapticBudgetConfig, EligibilityConfig), `dopamine.rs` (DopamineConfig), `reward.rs` (RewardConfig), `input.rs` (InputConfig), `output.rs` (OutputConfig)
**Problème :** Les champs `enabled: bool` existent encore dans les structs Rust et les `Default` impls avec `enabled: false`. Ils ne sont plus vérifiés dans la logique mais encombrent les structs et sont source de confusion.
**Solution :** Supprimer `enabled` de toutes les structs, et dans les `Default` impls, fournir des valeurs neutres pour les champs actifs.
**Notes :** Le TOML `config_v4_reward.toml` est déjà propre (pas de `enabled =`). Mais d'autres configs (config_v3_*.toml) peuvent encore avoir `enabled = true/false` → serde ignorera le champ manquant grâce à `#[serde(default)]`.

### F2. `PlasticityConfig` inutilisée ⬚
**Fichier :** `config.rs`
**Problème :** `PlasticityConfig` (reinforcement_rate, weakening_rate, etc.) est toujours dans `SimConfig` et le TOML (`[plasticity]`), mais n'est plus utilisée dans le pipeline. Le pipeline V4 utilise `StdpConfig` + `EligibilityConfig` + `DopamineConfig`.
**Constats dans le code :**
- `simulation.rs:step()` passe `&config.plasticity` à `stdp::update_plasticity_stdp_budget()` mais ce paramètre **n'est pas utilisé dans stdp.rs** (la signature l'accepte mais elle a été retirée dans la version actuelle — à vérifier)
- `stdp.rs` n'a plus de paramètre `PlasticityConfig` dans sa signature actuelle
**Solution :** Supprimer `PlasticityConfig` de `SimConfig`, retirer la section `[plasticity]` du TOML, et nettoyer les éventuelles références mortes.

### F3. `MetricsLog::record()` — signature incomplète ⬚
**Fichier :** `metrics.rs`
**Problème :** `record()` accepte `(tick, domain, zone_mgr, cumulative_reward, dopamine_level, decision, accuracy)` dans `simulation.rs` mais la signature dans `metrics.rs` est `(tick, domain, zone_mgr)` — les champs V4 (reward, dopamine, eligibility, decision, accuracy) existent dans `TickMetrics` mais ne sont pas remplis.
**Solution :** Ajouter les paramètres V4 à `record()` ou passer une struct `V4Metrics`.

### F4. CLI code dupliqué ⬚
**Fichier :** `stretch-cli/src/main.rs`
**Problème :** `run_v4_training()` dans le CLI duplique 90% de `setup_v4_training()` + la boucle principale de `run_with_observer()`. Le code spatial I/O selection (in0, in1, out0, out1) est écrit deux fois : dans le CLI et dans `Simulation::setup_v4_training()`.
**Solution :** Le CLI doit utiliser uniquement `sim.setup_v4_training()` + `run_with_observer()` comme le fait le viz.

---

## G. PLAN D'EXÉCUTION — Ordre de priorité

### Phase 1 : Dette technique résiduelle (F1-F4)
**Pré-requis avant toute optimisation.** Nettoyer le code pour que la base soit saine.

| Étape | Tâche | Fichiers | Effort |
|-------|-------|----------|--------|
| F1 | Supprimer `enabled: bool` des structs | config.rs, dopamine.rs, reward.rs, input.rs, output.rs | S |
| F2 | Supprimer `PlasticityConfig` | config.rs, simulation.rs, TOML | S |
| F3 | Compléter `MetricsLog::record()` V4 | metrics.rs, simulation.rs | S |
| F4 | Déduplication CLI | cli/main.rs | M |

### Phase 2 : Optimisations algorithmiques (C1-C6)
**Impact maximal pour effort minimal.** CPU pur, pas de dépendance externe.

| Étape | Tâche | Impact estimé | Fichiers | Effort |
|-------|-------|--------------|----------|--------|
| C3 | Hot edges list | PLAST: 83→15ms | stdp.rs, domain.rs | L |
| C2 | Skip STDP ψ inactives | PLAST: 15→8ms | stdp.rs | M |
| C4 | Budget incrémental | PLAST-D: 8→1ms | stdp.rs, domain.rs | M |
| C1 | Fired-only propagation | PROPAG: 12→3ms | propagation.rs, domain.rs | L |
| C5 | Skip dissip nœuds repos | dissip: 4.5→1ms | simulation.rs | M |
| C6 | Skip zones stables | zones: 12→3ms | zone.rs | S |

**Cible Phase 2 :** ~114ms → ~20-25ms/tick (×5-6x speedup, CPU seul)

### Phase 3 : GPU compute (D0-D5)
**Après la Phase 2, quand les algorithmes sont optimisés.** Le GPU accélère les kernels restants.

| Étape | Tâche | Fichiers | Effort |
|-------|-------|----------|--------|
| D0 | Ajouter wgpu + bytemuck + pollster | Cargo.toml | S |
| D1 | Module gpu.rs (init, buffers, dispatch) | gpu.rs (nouveau) | XL |
| D2 | Shader plasticity.wgsl | shaders/ (nouveau) | L |
| D3 | Shader propagation.wgsl | shaders/ (nouveau) | L |
| D4 | Shader budget.wgsl | shaders/ (nouveau) | M |
| D5 | Fallback CPU transparent | simulation.rs | M |

**Cible Phase 3 :** ~20ms → ~3-5ms/tick (×4-6x supplémentaire)

### Phase 4 : Framework (E1-E4)
**Après que les performances sont acquises.** Structurer pour la pérennité.

| Étape | Tâche | Fichiers | Effort |
|-------|-------|----------|--------|
| E1 | Réorg dossier shaders + gpu | structure | S |
| E2 | Documenter les briques réutilisables | README/docs | M |
| E3 | Traits d'abstraction (ComputeEngine, PlasticityRule) | simulation.rs | L |
| E4 | Config `[compute]` backend auto/cpu/gpu | config.rs | S |

---

## H. RÉSULTATS CIBLES

| Métrique | Avant (CPU) | Phase 2 (CPU opt) | Phase 3 (GPU) |
|----------|------------|-------------------|---------------|
| **Total ms/tick** | **114.19** | **~20-25** | **~3-5** |
| PLAST+STDP | 83.73 | ~8-12 | ~0.5-2 |
| PROPAG | 11.74 | ~2-3 | ~0.1-0.5 |
| zones | 11.98 | ~3-5 | ~3-5 (CPU) |
| dissip | 4.50 | ~1-2 | ~1-2 (CPU) |
| metrics | 2.09 | ~2 | ~2 (CPU) |
| **Speedup** | 1× | **~5-6×** | **~25-35×** |

| Qualité code | Avant | Après |
|-------------|-------|-------|
| `enabled` flags | 9 (dans structs) | 0 |
| Code mort | PlasticityConfig, CLI dupe | 0 |
| Backend GPU | aucun | wgpu (cross-platform) |
| Briques réutilisables | monolithique | 7 modules identifiés |
| Traits d'abstraction | aucun | 3 (ComputeEngine, PlasticityRule, ExperimentProtocol) |
