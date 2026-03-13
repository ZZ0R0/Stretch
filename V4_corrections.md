# V4 Corrections — Plan de refactoring complet

> **Baseline perf (50k nœuds, 577k arêtes, 200 ticks) : 10.81 ms/tick**
>
> Principes directeurs :
> 1. **Zéro option orpheline** — toutes les features V1→V4 sont activées, interdépendantes, fonctionnent en synergie.
> 2. **Abstraction haut niveau avant optimisation bas niveau** — restructurer données et algorithmes avant de micro-optimiser.

---

## A. DETTE TECHNIQUE — Features/configs orphelines à intégrer

### A1. Supprimer tous les flags `enabled: bool` ✅
9 sous-systèmes ont un flag `enabled: bool` indépendant (zones, consolidation, neuron_types, stdp, synaptic_budget, dopamine, reward, eligibility, input/output). Supprimés — tous les sous-systèmes sont toujours actifs. Les valeurs par défaut sont des valeurs neutres.

### A2. Supprimer `plasticity.rs` (code mort) ✅
Module entier jamais appelé depuis V3. Remplacé par `stdp.rs`.

### A3. Pipeline plasticité unifié ✅
Un seul pipeline dans `stdp.rs` : STDP→éligibilité→trois-facteurs→decay homéo→consolidation→budget.

### A4. Consolidation gatée dopamine ✅
Consolidation ne s'incrémente que si `dopamine_level >= threshold` en plus de `conductance >= threshold`.

### A5. `edge_defaults.decay` intégré ✅
Decay homéostatique des conductances vers baseline intégré au pipeline plasticité.

### A7. InputEncoder remplace stimuli classiques en V4 ✅
Si input est configuré, les `[[stimuli]]` classiques sont ignorés.

### A8. CLI utilise `setup_v4_training()` ✅
Code dupliqué dans le CLI supprimé, utilise la méthode partagée.

### A10. Ancien `compute_influences()` supprimé ✅
Seul le chemin caché (`compute_source_contribs` + `compute_influences_cached`) reste.

### A11. Métriques V4 complètes ✅
`MetricsLog::record()` reçoit et stocke toutes les métriques V4 (reward, dopamine, eligibility, decision, accuracy).

---

## B. ABSTRACTIONS HAUT NIVEAU — Optimisations architecturales

### B1. Persister le KD-tree dans Domain ✅
KD-tree stocké dans `Domain`, utilisé par `select_nearest_nodes()` et future spatialisation.

### B2. Table de voisinage pré-calculée + kernel_weight ✅
Chaque nœud a ses voisins entrants/sortants avec `kernel_weight` pré-calculé (élimine exp() par tick).
Remplace `adjacency` et `incoming_adjacency`.

### B4. CSR adjacence ✅
Deux tableaux contigus (offsets + données) au lieu de Vec<Vec<usize>>. Élimine 100k allocs heap.

### B5. Fired list (nœuds actifs) ✅
Maintien d'un `Vec<usize>` des nœuds actifs. Source contribs n'itère que les actifs (~5%).

### B7. Bitmap node_is_excitatory ✅
`Vec<bool>` pré-calculé. Évite de charger le Node complet en cache pour la propagation.

### B8. Arêtes éligibles seulement ✅
Liste d'arêtes avec éligibilité non-nulle. Seules celles-ci reçoivent la mise à jour dopaminergique.

---

## C. RÉSULTATS

| Métrique | Avant | Après |
|----------|-------|-------|
| ms/tick moyen | 10.81 | TBD |
| Propagation ms | 4.47 | TBD |
| Plasticité ms | 5.04 | TBD |
| flags `enabled` | 9 | 0 |
| Code mort (lignes) | ~200 | 0 |
| Code dupliqué CLI | ~100 lignes | 0 |
