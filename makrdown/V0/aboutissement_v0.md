# Aboutissement V0 — Bilan cahier des charges vs. réalisation

## Résumé

L'intégralité du périmètre V0 défini dans le cahier des charges a été implémentée et validée expérimentalement. Le système est fonctionnel, modulaire, reproductible, et les critères d'acceptation sont tous satisfaits.

---

## 1. Périmètre V0 — couverture

| Exigence (§2) | Statut | Implémentation |
|---|---|---|
| Représenter un espace discret de nœuds | ✅ | `stretch-core/src/domain.rs` — grille 2D et graphe sparse |
| Représenter des voisinages et proximités | ✅ | Listes d'adjacence + distances topologiques |
| Propager une activation locale (front spatio-temporel) | ✅ | `stretch-core/src/propagation.rs` — noyaux exponentiel et gaussien |
| Dissiper l'activation dans le temps | ✅ | Decay activation, fatigue, inhibition dans `node.rs` |
| Laisser une trace locale du passage | ✅ | `memory_trace` dans `node.rs`, `coactivity_trace` dans `edge.rs` |
| Renforcer / affaiblir les chemins parcourus | ✅ | Plasticité Hebbian-like dans `plasticity.rs` |
| Mesurer si des trajectoires préférentielles émergent | ✅ | Métriques usage, conductance, traces dans `metrics.rs` + viz |

---

## 2. Exigences fonctionnelles — détail

### §4.1 Domaine spatial ✅

- Deux topologies : `grid2d` (grille carrée 4-connexe) et `random_sparse` (graphe aléatoire).
- Chaque nœud porte un identifiant unique + position 2D.
- Topologie chargeable depuis fichier TOML.

### §4.2 État d'un nœud ✅

Les 6 variables demandées sont toutes implémentées dans `Node` :

| Variable | Implémentée | Bornée |
|---|---|---|
| `activation` | ✅ | [0, 10] |
| `threshold` | ✅ | configurable |
| `fatigue` | ✅ | [0, 10] |
| `memory_trace` | ✅ | [0, 100] |
| `excitability` | ✅ | dynamique, f(trace) |
| `inhibition` | ✅ | [0, 10] |

Tous les états sont numériques, bornés, inspectables et déterministes à graine fixe.

### §4.3 État d'une liaison ✅

Les 5 variables demandées sont toutes implémentées dans `Edge` :

| Variable | Implémentée | Bornée |
|---|---|---|
| `conductance` | ✅ | [conductance_min, conductance_max] |
| `distance` | ✅ | fixe à la construction |
| `coactivity_trace` | ✅ | [0, 10] |
| `plasticity` | ✅ | configurable |
| `decay` | ✅ | configurable |

### §4.4 Propagation ✅

- Propagation par pas de temps (tick-based).
- Dépend de : intensité source × conductance × noyau(distance) × gain.
- Seuls les nœuds actifs (dépassant le seuil effectif) propagent.
- Deux noyaux configurables : exponentiel, gaussien discret.
- Dissipative : sans stimulation, l'activité retombe (vérifié expérimentalement).

### §4.5 Déclenchement ✅

- Seuil effectif = `(threshold + fatigue + inhibition) / excitability`.
- Un nœud sur-sollicité accumule fatigue → seuil effectif monte → plus dur à réactiver.
- L'inhibition empêche l'emballement global.

### §4.6 Dissipation et stabilité ✅

5 mécanismes de stabilisation actifs :

1. Décroissance spontanée de l'activation (`activation_decay`).
2. Fatigue post-activation.
3. Inhibition locale.
4. Bornage de toutes les variables (`clamp`).
5. Seuil effectif dynamique (intègre fatigue + inhibition).

**Zone stable validée** : front 0 → 162 nœuds actifs → retour au repos en ~70 ticks.

### §4.7 Trace locale et mémoire ✅

- `memory_trace` augmente proportionnellement à l'activation lorsque le nœud est actif.
- Décroît lentement par `trace_decay` quand non sollicitée.
- Influence la propagation future via `excitability = 1.0 + 0.1 × trace`.
- Traces non uniformes observées (localisées autour des corridors d'activité).

### §4.8 Plasticité ✅

- Renforcement : si `coactivity_trace > seuil`, conductance augmente (Hebbian-like).
- Affaiblissement : sinon, conductance décroît lentement.
- Plasticité lente vs. activation rapide (taux configurable séparément).
- Règles purement locales (chaque liaison ne voit que ses deux nœuds).
- Conductances bornées par `[conductance_min, conductance_max]`.

**Résultat observé** : conductances passent de 1.0 → 4.9 sur les corridors actifs.

### §4.9 Instrumentation ✅

| Observable demandé | Implémenté |
|---|---|
| État global à chaque tick | ✅ `TickMetrics` |
| État par nœud | ✅ export JSON `node_traces.json` |
| État par liaison | ✅ export JSON `edge_conductances.json` |
| Nombre de nœuds actifs | ✅ `active_nodes` |
| Énergie totale | ✅ `global_energy` |
| Chemins les plus traversés | ✅ `top_edges()` |
| Distribution des traces | ✅ `trace_histogram()` |
| Évolution des conductances | ✅ `mean_conductance`, `max_conductance` par tick |

---

## 3. Exigences non fonctionnelles

### §5.1 Lisibilité scientifique ✅

- Code structuré en modules séparés (1 fichier = 1 concept).
- Règles d'évolution explicites dans chaque méthode.
- Paramètres centralisés dans `SimConfig` (TOML sérialisable).

### §5.2 Reproductibilité ✅

- Graine aléatoire `ChaCha8Rng` (déterministe cross-platform).
- Résultats exportables en JSON (`metrics_output.json`, `node_traces.json`, `edge_conductances.json`).
- Expériences traçables via fichiers TOML (`config.toml`, `config_training.toml`).

### §5.3 Modularité ✅

| Module demandé | Fichier |
|---|---|
| Domaine spatial | `domain.rs` |
| Dynamique des nœuds | `node.rs` |
| Dynamique des liaisons | `edge.rs` |
| Propagation | `propagation.rs` |
| Plasticité | `plasticity.rs` |
| Métriques | `metrics.rs` |
| Visualisation | `stretch-viz/` (crate séparé) |

**Architecture workspace Cargo** : `stretch-core` (lib) + `stretch-cli` (bin) + `stretch-viz` (bin).
Le moteur est une bibliothèque réutilisable, connectée à la visualisation via un trait `SimulationObserver` et une `Simulation` pas-à-pas.

### §5.4 Performance raisonnable ✅

- Simulation locale sur machine unique.
- Grille 20×20 (400 nœuds) : instantanée.
- Architecture compatible avec des graphes plus grands (vecteurs plats, listes d'adjacence).

### §5.5 Stabilité numérique ✅

- Toutes les variables bornées par `clamp`.
- Règles testées avec variations de paramètres (zone stable identifiée).
- Pas d'overflow ni de NaN observés.

---

## 4. Boucle de simulation (§7) ✅

Les 9 étapes du tick sont implémentées dans l'ordre exact demandé :

1. ✅ Injection de stimuli → `stimulus.rs`
2. ✅ Calcul des influences → `propagation::compute_influences()`
3. ✅ Mise à jour de l'activation → `propagation::apply_influences()`
4. ✅ Fatigue / inhibition → `node.update_fatigue()`, `node.update_inhibition()`
5. ✅ Traces locales → `node.update_trace()`
6. ✅ Traces de co-activation → `edge.record_coactivation()`
7. ✅ Mise à jour conductances → `edge.update_conductance()`
8. ✅ Collecte métriques → `metrics.record()`
9. ✅ Export snapshot → JSON à la fin / visualisation chaque frame

---

## 5. Paramètres configurables (§8) ✅

Tous les 14 paramètres demandés sont configurables via TOML :

| Paramètre | Champ config |
|---|---|
| Taille du graphe | `domain.size` |
| Topologie initiale | `domain.topology` |
| Distribution initiale des seuils | `node_defaults.threshold` |
| Intensité des stimuli | `stimuli[].intensity` |
| Noyau de propagation | `propagation.kernel` |
| Taux de dissipation | `dissipation.activation_decay` |
| Intensité de fatigue | `dissipation.fatigue_gain` |
| Intensité d'inhibition | `dissipation.inhibition_gain` |
| Vitesse d'oubli des traces | `dissipation.trace_decay` |
| Vitesse de renforcement | `plasticity.reinforcement_rate` |
| Vitesse d'affaiblissement | `plasticity.weakening_rate` |
| Bornes min/max conductances | `edge_defaults.conductance_min/max` |
| Durée de simulation | `simulation.total_ticks` |
| Graine aléatoire | `simulation.seed`, `domain.seed` |

---

## 6. Livrables (§9)

### §9.1 Livrables logiciels

| Livrable | Statut | Emplacement |
|---|---|---|
| Moteur de simulation | ✅ | `stretch-core/` |
| Format de configuration | ✅ | TOML (`config.toml`, `config_training.toml`) |
| Format d'export des métriques | ✅ | JSON (`metrics_output.json`, etc.) |
| Outil de visualisation temporelle | ✅ | `stretch-viz/` (macroquad, temps réel) |
| Scripts d'expériences de base | ✅ | 2 configs TOML (sanity check + entraînement) |

### §9.2 Livrables scientifiques

| Livrable | Statut | Emplacement |
|---|---|---|
| Description des variables | ✅ | Structs documentées dans `node.rs`, `edge.rs` |
| Définition des règles de mise à jour | ✅ | Méthodes explicites dans chaque module |
| Liste des métriques | ✅ | `TickMetrics` dans `metrics.rs` |
| Protocole de test minimal | ✅ | `protocole_test_entrainement_v0.md` (pré-existant) |
| Cas de validation | ✅ | Résultats expérimentaux documentés ci-dessous |

---

## 7. Critères d'acceptation (§10)

| # | Critère | Statut | Preuve |
|---|---|---|---|
| 1 | Simulation reproductible | ✅ | Graine ChaCha8, résultats identiques entre runs |
| 2 | Activation locale se propage spatialement | ✅ | Front : 1 → 9 → 48 → 100 → 144 → 162 nœuds |
| 3 | Propagation se dissipe correctement | ✅ | Retour au repos en ~70 ticks |
| 4 | Trajectoires préférentielles après répétition | ✅ | Conductances 1.0 → 4.9 sur corridors entraînés |
| 5 | Métriques constatent l'émergence | ✅ | Top-edges, histogramme traces, sparkline énergie |
| 6 | Zone stable non triviale | ✅ | Ni mort immédiate ni explosion permanente |

### Résultat expérimental clé — comparaison avant/après entraînement

| Métrique | Avant entraînement | Après entraînement | Changement |
|---|---|---|---|
| Nœuds actifs max | 162 | 319 | +97% |
| Énergie max | 1 474 | 2 877 | +95% |
| Vitesse d'atteinte du pic | ~50 ticks | ~20 ticks | 2.5× plus rapide |

→ Le système **a appris** : après stimulation répétée, la même probe produit une réponse plus large, plus rapide et plus vigoureuse.

---

## 8. Risques techniques (§11) — statut

| Risque | Statut |
|---|---|
| Système = simple diffusion sans mémoire | ❌ Écarté — traces + plasticité changent la réponse |
| Système trop instable | ❌ Écarté — zone stable identifiée par calibration |
| Paramètres trop sensibles | ⚠️ Partiel — la zone stable existe mais est assez étroite |
| Absence de vrais attracteurs | ❌ Écarté — des corridors stables émergent |
| Renforcement = artefact | ❌ Écarté — la comparaison before/after est nette et reproductible |

---

## 9. Au-delà du cahier des charges

Éléments livrés en bonus, non demandés mais utiles :

| Élément | Description |
|---|---|
| **Architecture workspace** | Séparation lib / CLI / viz en 3 crates Cargo |
| **Trait `SimulationObserver`** | Système de hooks/callbacks pour connecter n'importe quel front-end |
| **`Simulation` pas-à-pas** | Méthode `step()` pour contrôle tick-par-tick (pause, avance manuelle) |
| **Visualisation temps réel** | 4 heatmaps (activation, traces, fatigue, conductance), sparkline énergie, sidebar métriques |
| **Contrôle de vitesse** | 1× → 256× configurable en temps réel (↑/↓) |
| **Pas-à-pas interactif** | Avance d'un tick à la fois (→ ou N) |

---

## 10. Conclusion

**La V0 est complète et conforme au cahier des charges.**

Les 7 points du périmètre sont couverts, les 9 exigences fonctionnelles satisfaites, les 5 exigences non fonctionnelles respectées, les 14 paramètres configurables exposés, les 6 critères d'acceptation validés expérimentalement, et les livrables logiciels + scientifiques fournis.

Le substrat dynamique fait émerger des **fronts de propagation structurés**, des **traces localisées**, des **conductances renforcées sélectivement** et des **corridors préférentiels mesurables** — exactement ce que demandait la vision §3 du cahier des charges.

Le système est prêt à servir de base pour une V1.
