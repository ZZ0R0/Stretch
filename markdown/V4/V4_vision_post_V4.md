# Vision post-V4 — Vers la V5

## Ce que la V4 laisse en héritage

### Infrastructure résolue

La V4, à travers ses trois phases de corrections, a construit un **moteur de simulation scalable** :

- **Architecture GPU-First** : 11 shaders WGSL, single submit/tick, zéro round-trip CPU↔GPU
- **Performance** : 0.3 ms/tick (50k), 24.6 ms/tick (5M) — 100× plus rapide que V3
- **Mémoire** : `compact_for_gpu()` réduit le RSS de 45 GB à 4 GB pour 5M nœuds
- **Scaling** : testé de 50k à 5M nœuds (57M arêtes) sans crash ni dégradation
- **Portabilité** : wgpu supporte Vulkan, Metal, DX12 — fonctionne sur les GPU NVIDIA RTX et les iGPU AMD
- **Pipeline de plasticité complet** : STDP → éligibilité → règle des 3 facteurs → homéostasie → consolidation → budget synaptique

### Questions ouvertes (non résolues)

1. **Le réseau apprend-il réellement ?** — Pas encore prouvé. L'accuracy 100% à 50k est un artefact du biais topologique.
2. **Les conductances forment-elles des chemins ?** — Pas de preuve. Modifications faibles (+0.9%) et symétriques.
3. **Les paramètres scalent-ils ?** — Non. Accuracy = 50% (chance) à 500k et 5M.

---

## Axes prioritaires pour la V5

### Axe 1 : Preuve d'apprentissage (priorité absolue)

Le but de la V5 est de **prouver que le réseau est capable d'apprentissage véritable** — pas juste de discrimination topologique.

#### 1.1 Test anti-biais topologique

Placer les I/O dans une configuration où la **topologie ne résout pas** la tâche :
- **Config symétrique** : inputs et outputs à distances égales. Ex : 4 groupes sur un carré, input-0 en (25, 50), input-1 en (75, 50), output-0 en (50, 25), output-1 en (50, 75). Distances égales → la géométrie ne peut pas discriminer.
- **Config d'association inversée** : garder la config actuelle mais associer input-0 → output-1 (la sortie la plus éloignée). Si le réseau apprend à re-router le signal via des chemins renforcés, c'est une preuve d'apprentissage.
- **Config croisée** : pré-entraîner avec une association, puis inverser. Mesurer le temps de ré-apprentissage.

#### 1.2 Métriques de preuve

Définir des métriques quantitatives de formation de chemin :
- **Conductance directionnelle** : somme des conductances le long du plus court chemin input-0→output-0 vs input-0→output-1
- **Index de cohérence topologique** : corrélation entre la conductance d'une arête et sa position sur un chemin I/O
- **Carte de chaleur des conductances** : visualiser les modifications dans l'espace 3D pour identifier les routes

#### 1.3 Baseline rigoureux

- **Random baseline** : réseau avec conductances aléatoires (pas d'apprentissage). Accuracy attendue = 50%.
- **Topologie-only baseline** : réseau figé (pas de plasticité, conductances = 1.0). Mesurer l'accuracy due uniquement à la géométrie.
- **Full learning** : réseau avec plasticité active. Comparer à la baseline topologie.

### Axe 2 : Calibration multi-échelle

Les paramètres ne scalent pas de 50k à 5M. La V5 doit adopter des **hyperparamètres adaptatifs** :

| Paramètre | V4 (fixe) | V5 (proposition adaptative) | Rationale |
|---|---|---|---|
| `group_size` | 50 | `max(50, √n)` | Stimulus proportionnel à la taille du réseau |
| `propagation.gain` | 0.8 | `0.8 × log(n) / log(50000)` | Compenser l'atténuation par la distance |
| `eligibility.decay` | 0.85 | 0.95–0.98 | Couvrir le read_delay (voir §4 instabilités) |
| `edge_defaults.decay` | 0.0001 | `0.0001 / (n / 50000)` | Réduire l'érosion pour les grands réseaux |
| `spatial_lambda` | 0.15 | `0.15 × extent / √n` | Portée dopamine proportionnelle |
| `target_activity` | 0.3 | `min(0.3, 5000 / n)` | Cible réaliste (~5000 nœuds actifs) |

### Axe 3 : Dynamique d'activité soutenue

Le « flash & die » empêche la formation de traces STDP robustes. Solutions possibles :

1. **Decay adaptatif** : `activation_decay = base_decay × (1 - feedback_factor × local_activity)`. Les zones actives maintiennent mieux leur activité.
2. **Connexions récurrentes** : boucles de rétroaction locales qui soutiennent l'activité au-delà de l'injection.
3. **Injection étendue** : augmenter `presentation_ticks` de 5 à 15-20 pour donner plus de temps à la propagation.
4. **Réverbération** : arêtes bidirectionnelles avec gain légèrement > 1.0 dans les zones proches de l'input, créant des micro-oscillations locales auto-entretenues.

### Axe 4 : Outils diagnostiques

Pour avancer efficacement, il faut des **outils de compréhension**, pas seulement d'exécution :

1. **Carte de conductance 3D** : visualiser les arêtes renforcées/affaiblies dans l'espace, colorées par ΔC
2. **Path tracer** : pour un input donné, tracer les chemins de conductance maximale de input → output
3. **Timeline d'éligibilité** : pour un sous-ensemble d'arêtes, tracer e(t) et C(t) au cours des trials
4. **Analyse de cluster** : identifier les groupes d'arêtes co-renforcées (assemblées de Hebb)

---

## Risques et conditions de passage

### Gate V4 → V5

| Condition | Statut |
|---|---|
| Pipeline plasticité complet (STDP → élig → 3-facteurs → conso) | ✅ |
| Architecture GPU-First fonctionnelle | ✅ |
| Scaling 500k+ vérifié (pas de crash) | ✅ |
| Preuve d'apprentissage au-delà du biais topologique | ❌ **Bloquant** → cible V5 |
| Tests de non-régression stables | ✅ (2/2 tests) |

### Risques V5

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| Le réseau ne peut pas apprendre (architecture fondamentalement inadaptée) | Moyenne | Critique | Tests anti-biais rapides avant gros investissement |
| Le scaling des paramètres est non-trivial (pas de formule simple) | Haute | Sévère | Grid search automatisé sur GPU (rapide grâce à V4) |
| La dynamique soutenue crée de l'instabilité (explosion d'activité) | Moyenne | Modéré | PID + clamps existants + tests incrémentaux |
| Les outils diagnostiques prennent trop de temps | Basse | Faible | Implémenter par étapes, le plus simple d'abord |

---

## Résumé

> La V4 a donné au projet ses **jambes** (GPU-First, scaling, performance).
> La V5 doit lui donner son **cerveau** (preuve d'apprentissage, calibration multi-échelle, dynamique soutenue).
>
> **Priorité V5** : ne pas optimiser davantage l'infra — prouver que le modèle biologique fonctionne.
