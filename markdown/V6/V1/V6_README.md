# V6 — Signal à longue portée sous contrainte de sparsité

## Contexte

La V5.x a mis en place un système fonctionnel de propagation par réseaux KNN 3D, STDP
trois-facteurs à fenêtre d'éligibilité modulée par la dopamine, régulation PID par zones,
et tâches anti-biais topologique. Le système fonctionne sans contrainte de sparsité.

## Problème V6

Quand on impose une **contrainte de sparsité** (max 5% de neurones actifs par tick),
le front d'onde de signal se retrouve piégé :

- À chaque hop, la propagation active ~10× plus de neurones via le fan-out KNN
- La compétition de sparsité sélectionne les neurones les **plus activés** — ceux au
  centre du front (convergence de multiples sources), pas ceux à la frontière
- Le front est **aspiré vers le centre** au lieu d'avancer vers l'output
- Le signal ne parcourt jamais la distance input→output (~60 unités, 15-20 hops)
- **Résultat** : scores = [0, 0], aucune récompense, pas d'apprentissage

## Solution V6 : Approche A + C

### A — Sparsité à front d'onde (Wavefront-aware sparsity)

Un mécanisme de **bonus de nouveauté** dans la compétition de sparsité. Les neurones
récemment activés pour la première fois (= la frontière) gagnent la compétition par
rapport aux neurones déjà actifs depuis plusieurs ticks (= le centre/corps).

$$\text{score}_i = \phi_i \times \left(1 + \lambda_{\text{nov}} \times \frac{\max(0, W - \tau_i)}{W}\right)$$

- τ_i = nombre de ticks depuis la première activation du neurone dans ce trial
- W = fenêtre de nouveauté (configurable, ex: 10 ticks)
- λ_nov = gain de nouveauté (configurable, ex: 2.0 → les neurones frais ont score ×3)

**Effet** : le corps de l'onde perd son avantage compétitif au fil des ticks, la
frontière avance naturellement — comme une vraie onde de propagation.

### C — Dynamiques modulées par la dopamine

En phase de recherche (dopamine basse / pas de récompense récente), les paramètres
dynamiques sont relâchés pour donner plus de chance au signal d'explorer :

- `reverb_gain` augmenté → activité plus soutenue
- `activation_decay` réduit → dissipation plus lente
- Budget de sparsité légèrement relâché

En phase d'exploitation (dopamine haute), retour aux paramètres stricts.

$$\text{reverb}_{\text{eff}} = r_{\min} + (r_{\max} - r_{\min}) \times \sigma\left(\frac{\theta - \bar{d}_{\text{recent}}}{\kappa}\right)$$

## Axes V6

| Axe | Composant | Priorité |
|-----|-----------|----------|
| 1 | Sparsité à front d'onde (shader + CPU) | P0 |
| 2 | Modulation par dopamine (reverb, decay adaptatifs) | P1 |
| 3 | Configuration, intégration pipeline, tests | P0 |

## Livrables

| ID | Livrable | Axe |
|----|----------|-----|
| L1 | `sparsity.rs` — logique CPU sparsité + compétition wavefront | 1 |
| L2 | `sparsity.wgsl` — shader GPU sparsité | 1 |
| L3 | `SparsityConfig` dans config.rs | 3 |
| L4 | Intégration dans le pipeline GPU (`run_full_tick`) | 3 |
| L5 | Intégration dans le pipeline CPU (`step_cpu`) | 3 |
| L6 | `config_v6.toml` — configuration de référence | 3 |
| L7 | Buffer `first_activation_tick` (GPU + CPU) | 1 |
| L8 | Modulation reverb/decay par dopamine dans `apply_and_dissipate.wgsl` | 2 |
| L9 | Test de cohérence CPU/GPU pour la sparsité | 3 |
