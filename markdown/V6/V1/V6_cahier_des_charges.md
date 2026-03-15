# V6 — Cahier des charges

## Exigences Fonctionnelles (EF)

### EF-1 : Compétition de sparsité globale
Le système doit limiter le nombre de neurones actifs à une fraction configurable
(`max_active_fraction`) du total à chaque tick. Les neurones non sélectionnés
voient leur activation suppressée (multipliée par `suppress_factor`).

### EF-2 : Bonus de nouveauté (wavefront)
La compétition de sparsité utilise un score pondéré :
```
score_i = activation_i × (1 + novelty_gain × max(0, novelty_window - age_i) / novelty_window)
```
où `age_i` = ticks écoulés depuis la première activation du neurone dans le trial courant.

### EF-3 : Tracking per-neurone du tick de première activation
Un buffer `first_activation_tick` (u32 par neurone) est maintenu, initialisé à
`0xFFFFFFFF` (jamais activé). Mis à jour quand un neurone dépasse son seuil effectif
pour la première fois dans un trial. Remis à zéro au reset de trial.

### EF-4 : Reset du buffer de première activation au trial start
Au début de chaque trial (quand `reset_activations = true`), le buffer
`first_activation_tick` est réinitialisé (clear GPU ou memset CPU).

### EF-5 : Suppression douce des neurones non-sélectionnés
Les neurones qui perdent la compétition ne sont pas forcément mis à zéro.
Un facteur `suppress_factor ∈ [0.0, 1.0]` contrôle la suppression :
- 0.0 = suppression totale (dur : activation = 0)
- 0.5 = suppression douce (activation × 0.5)
- 1.0 = pas de suppression (sparsité = comptage seul, pas d'enforcement)

### EF-6 : Modulation de la réverbération par la dopamine
Le gain de réverbération effectif est modulé par le niveau de dopamine :
```
reverb_eff = reverb_min + (reverb_max - reverb_min) × σ((θ_dopa - dopa_level) / κ)
```
En phase de recherche (dopamine basse), la réverbération est augmentée pour
soutenir la propagation du signal.

### EF-7 : Modulation du decay par la dopamine
Le decay effectif est modulé symétriquement :
```
decay_eff = decay_base × (1 - dopa_decay_mod × σ((θ_dopa - dopa_level) / κ))
```
En phase de recherche, le decay est réduit (signal persiste plus longtemps).

### EF-8 : Pipeline GPU — insertion du pass de sparsité
Le pass de sparsité s'insère **après** `apply_and_dissipate` (Phase 7) et
**avant** la réverbération (Phase 7b). Ordre :
1. Phase 7 : apply_and_dissipate (calcule les activations brutes)
2. **Phase 7a : sparsity** (sélection top-K, suppression)
3. Phase 7b : reverberation (nourrit les survivants)

### EF-9 : Configuration dédiée `v6_sparsity`
Section de config TOML :
```toml
[v6_sparsity]
enabled = true
max_active_fraction = 0.05
suppress_factor = 0.0
novelty_gain = 2.0
novelty_window = 10
```

### EF-10 : Configuration `v6_dopa_modulation`
Section de config TOML :
```toml
[v6_dopa_modulation]
enabled = true
reverb_min = 0.05
reverb_max = 0.30
decay_mod_strength = 0.3
dopa_threshold = 0.15
dopa_kappa = 0.05
```

## Exigences Non-Fonctionnelles (ENF)

### ENF-1 : Compatibilité descendante
Quand `v6_sparsity.enabled = false` et `v6_dopa_modulation.enabled = false`,
le système se comporte exactement comme V5.3. Tous les tests V5 existants
continuent de passer.

### ENF-2 : Performance GPU
L'ajout du pass de sparsité ne doit pas augmenter le temps par tick de plus de
20% par rapport à V5.3 (sur GPU).

### ENF-3 : Cohérence CPU/GPU
Le résultat de la sparsité doit être identique (à la précision flottante près)
entre les chemins CPU et GPU.

### ENF-4 : Prévision pour les neurones de contrôle de groupe
La sparsité opère globalement pour V6, mais l'architecture doit permettre 
une future extension à une sparsité par zone (V7+), où chaque zone a son
propre budget et ses propres paramètres de modulation.

## Critères d'acceptance (CA)

### CA-1 : Le signal atteint l'output
Avec `max_active_fraction = 0.05`, `novelty_gain = 2.0`, la simulation V6
en mode Symmetric doit produire des scores de readout non-nuls (`max(scores) > 0.1`)
dans au moins 80% des trials.

### CA-2 : La sparsité est respectée
Le nombre moyen de neurones actifs par tick ne dépasse pas
`max_active_fraction × num_nodes × 1.05` (marge de 5%).

### CA-3 : L'apprentissage fonctionne
Accuracy > 55% (significativement au-dessus du hasard de 50%) sur 5 seeds.

### CA-4 : Tests V5 préservés
10/10 tests V5.2 existants passent sans modification.

### CA-5 : Build propre
0 erreurs, 0 warnings (cargo build --release, cargo clippy).
