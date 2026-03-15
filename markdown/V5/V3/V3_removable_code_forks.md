# V5.3 — Code Forks Supprimables

> Recensement des embranchements de code (forks) qui peuvent être supprimés ou unifiés
> pour simplifier la codebase avant ou pendant la V5.3.

---

## 1. Contexte

La V5.2 a hérité de plusieurs chemins de code dupliqués ou devenus obsolètes.
Ce document identifie les forks supprimables pour réduire la complexité et
éliminer les sources de divergence CPU/GPU.

---

## 2. Forks identifiés

### 2.1 Shader fusionné `apply_and_dissipate.wgsl`

**Type** : code dupliqué entre CPU et GPU avec sémantique différente.

| Aspect | CPU (simulation.rs + sustained.rs) | GPU (apply_and_dissipate.wgsl) |
|--------|-----|-----|
| Apply influences | `propagation::apply_influences()` | Intégré dans le shader |
| Fatigue update | Post-réverbération | Pré-réverbération (dans le shader) |
| Inhibition update | Post-réverbération | Pré-réverbération |
| Trace update | Post-réverbération | Pré-réverbération |
| Decay/dissipation | Fonction séparée dans `sustained.rs` | Conditionnel dans le shader |

**Impact** : ~5pp de gap CPU/GPU dû au timing différent.

**Action V5.3** : Séparer en 2 shaders :
- `apply_influences.wgsl` — accumulation, seuil, activation
- `dissipation.wgsl` — fatigue, inhibition, trace, decay

**Fichiers** : `stretch-core/shaders/apply_and_dissipate.wgsl` (~120 lignes) → 2 fichiers.

### 2.2 Double chemin `step_gpu()` / `step_cpu()` résiduel

**Type** : fork historique partiellement unifié.

La V5.orch a extrait `process_readout()` et unifié la logique métier, mais `step_gpu()` et `step_cpu()` existent toujours comme méthodes distinctes dans `simulation.rs`. L'orchestrateur les appelle via un `match` sur le backend.

**Action possible** : transformer en vrai trait `ComputeBackend` (tel que spécifié dans `V5.orch_architecture_systeme.md`) avec `GpuBackend` et `CpuBackend` comme implémentations.

**Risque** : moyen — le refactoring est non-trivial car `step_cpu()` et `step_gpu()` accèdent directement aux champs de `Simulation`.

**Priorité** : basse — le fork actuel est gérable et ne cause pas de divergence fonctionnelle (la logique métier est unifiée).

### 2.3 Jitter hash CPU vs GPU

**Type** : implémentation divergente du même concept.

| | CPU | GPU |
|---|-----|-----|
| Type | u64 LCG (Knuth) | u32 mul-add |
| Seed | `node_index ^ tick` | `node_id ^ tick` |
| Distribution | [0, 1) f64 → f32 | [0, 1) f32 directe |

**Impact** : bruit stochastique différent entre CPU et GPU. Pas de biais systématique mais empêche la reproduction bit-identical.

**Action possible** : unifier sur l'algorithme GPU (u32 mul-add) côté CPU.

**Priorité** : très basse — le non-déterminisme GPU (ordonnancement atomique) rend la reproduction bit-identical impossible de toute façon.

### 2.4 Budget : incrémental (CPU) vs fresh (GPU)

**Type** : algorithme différent pour le même résultat.

| | CPU (stdp.rs) | GPU (budget.wgsl) |
|---|---------------|-------------------|
| Quand | Après chaque edge modifié | Après toutes les modifications, par nœud source |
| Somme | Incrémentale (partielle) | Globale (atomicAdd sur tous les edges) |
| Scaling | Immédiat | En une passe séparée |

**Impact** : le GPU est plus « précis » (somme complète → scaling exact). Le CPU peut appliquer des scalings intermédiaires qui se composent légèrement différemment.

**Action possible** : aligner le CPU sur l'algorithme GPU (somme globale → scaling global).

**Priorité** : basse — l'impact mesuré est <1pp.

### 2.5 Éligibilité : active set (CPU) vs tous les edges (GPU)

**Type** : optimisation CPU qui n'existe pas sur GPU.

| | CPU | GPU |
|---|-----|-----|
| Edges traités | `active_edge_set` (snapshot) | Tous les 577k edges |
| Decay éligibilité | Seulement sur edges actifs | Sur tous les edges |

**Impact** : fonctionnellement équivalent — les edges inactifs ont $\psi = 0$ et $e$ décroît vers 0 de toute façon. Mais le CPU est plus rapide (ne traite que ~5-15% des edges) tandis que le GPU a un parallélisme massif qui compense.

**Action** : aucune — les deux approches sont correctes et adaptées à leur architecture.

---

## 3. Matrice de priorité

| Fork | Impact | Effort | Priorité V5.3 |
|------|--------|--------|----------------|
| Shader fusionné | ~5pp | Modéré | **HAUTE** |
| step_gpu/step_cpu | Maintenance | Élevé | Basse |
| Jitter hash | <0.1pp | Faible | Très basse |
| Budget incrémental | <1pp | Modéré | Basse |
| Éligibilité active set | 0pp | N/A | Aucune |

---

## 4. Recommandation

**V5.3 devrait prioritairement** : séparer le shader fusionné (2.1). C'est le seul fork avec un impact mesurable significatif (~5pp) et il est la première cause du gap CPU↔GPU résiduel.

Les autres forks sont des optimisations de cleanup qui ne justifient pas le risque de régression.
