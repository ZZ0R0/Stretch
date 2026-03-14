# Aboutissement V4

## 1. Bilan des objectifs

La V4 avait trois objectifs principaux, réalisés en trois phases de corrections :

### V4.1 — Refactoring & unification (V4_corrections.md)

| Objectif | Résultat |
|---|---|
| Supprimer les 9 flags `enabled: bool` orphelins | ✅ Supprimés — tous les sous-systèmes toujours actifs |
| Supprimer le code mort (`plasticity.rs`) | ✅ ~200 lignes éliminées |
| Pipeline plasticité unifié (STDP→éligibilité→3-facteurs→homéo→conso→budget) | ✅ Un seul pipeline dans `stdp.rs` |
| Consolidation gatée par dopamine | ✅ Requiert `dopamine ≥ threshold` |
| Decay homéostatique intégré au pipeline | ✅ `edge_defaults.decay` appliqué |
| InputEncoder remplace stimuli classiques | ✅ |
| CSR pour arêtes entrantes + kernel_weights pré-calculés | ✅ Élimine `exp()` par tick |
| KD-tree persisté dans Domain | ✅ Réutilisé pour requêtes spatiales |
| Fired list (~5% nœuds actifs seulement) | ✅ |
| Bitmap `node_is_excitatory` | ✅ Évite chargement du Node complet |

### V4.2 — Optimisations algorithmiques + GPU compute (V4_corrections_2.md)

| Objectif | Résultat |
|---|---|
| Passage au GPU via wgpu (Vulkan/Metal/DX12) | ✅ Backend `auto`/`cpu`/`gpu` |
| Hot edges list (arêtes éligibles seulement) | ✅ ~5-10% des arêtes traitées |
| Budget synaptique incrémental | ✅ `running_totals` par nœud |
| Dissipation : skip nœuds à l'équilibre | ✅ Bitmap `node_needs_update` |
| Cible : < 5 ms/tick (50k nœuds) | ✅ Atteint 0.3 ms/tick en GPU-First |

### V4.3 — Architecture GPU-First (V4_corrections_3.md)

| Objectif | Résultat |
|---|---|
| Tout l'état mutable (nœuds + arêtes) vit sur le GPU | ✅ |
| 11 shaders WGSL couvrant le pipeline complet | ✅ injection, zones, source_contribs, propagation, apply_influences, dissipation, plasticity_stdp, budget_sum, budget_normalize, readout, metrics |
| Single `queue.submit()` par tick, zéro readback sauf décision | ✅ |
| Transferts/tick : ~200 B upload, ~64 B download (vs 18.5 MB avant) | ✅ |
| Cible : < 3 ms/tick (50k nœuds) | ✅ **0.3 ms/tick** atteint |

### Scaling & robustesse (travail post-corrections)

| Objectif | Résultat |
|---|---|
| Support des limites GPU réelles (`adapter.limits()`) | ✅ Remplace `wgpu::Limits::default()` |
| Dispatch 2D pour >65535 workgroups | ✅ 12 shaders mis à jour |
| Optimisation mémoire (`compact_for_gpu`) | ✅ RSS 5M : 9.5 GB pic → 4.0 GB stable |
| Fonctionnement à 500k nœuds | ✅ ~5.7M arêtes, 2.0 ms/tick |
| Fonctionnement à 5M nœuds | ✅ ~57M arêtes, 24.6 ms/tick, 4.0 GB RAM |

---

## 2. Effet réel de la dopamine

### Mécanisme implémenté

La dopamine suit un modèle tonic/phasic spatialisé :
- **Tonic** : baseline = 0.1, maintient une plasticité de fond
- **Phasic** : bouffée positive (reward correct) ou négative (reward incorrect), décroît avec `phasic_decay = 0.15`
- **Spatialisation** : le signal dopaminergique est modulé par la distance au reward center, avec `spatial_lambda = 0.15`
- **Effet** : `ΔC = η × δ_d × e_ij` (règle des trois facteurs)

### Observations à 50k nœuds

| Métrique | Valeur (5000 ticks, 161 trials) |
|---|---|
| Dopamine finale | 0.625 |
| Récompense cumulée | 161.0 (100% correct) |
| Conductance moyenne | 1.009 (décalage +0.9% du baseline) |
| Arêtes > 1.05 | 48 556 / 577 488 (8.4%) |
| Arêtes < 0.95 | 49 029 / 577 488 (8.5%) |
| Max éligibilité | 0.032 |

### Analyse

L'effet dopaminergique est **mesurable mais faible** : seulement +0.9% de déplacement moyen de conductance. Le renforcement est quasi-symétrique (autant d'arêtes renforcées qu'affaiblies). Ceci est cohérent avec le biais topologique qui rend l'apprentissage par conductance non-nécessaire pour la discrimination (voir §8).

### Observations à 5M nœuds

La conductance moyenne chute à 0.304 (vs baseline 1.0) après 1000 ticks. Ceci indique que le **decay homéostatique domine massivement** l'apprentissage à grande échelle — les arêtes distantes des I/O ne reçoivent aucun signal dopaminergique et sont érodées uniformément.

---

## 3. Effet réel du reward

### Mécanisme

- Reward positif (+1.0) quand la décision du readout correspond au target
- Reward négatif (-0.5) en cas d'erreur
- Le reward déclenche un burst dopaminergique phasique, qui modifie les conductances via les traces d'éligibilité

### Observations

| Échelle | Accuracy | Interprétation |
|---|---|---|
| 50k nœuds | **100% (161/161)** | Discrimination parfaite — mais due au biais topologique, pas au reward |
| 500k nœuds | **50% (chance)** | Pas d'apprentissage observé avec les paramètres 50k |
| 5M nœuds | **50% (16/32, chance)** | Idem — paramètres non calibrés pour cette échelle |

Le reward fonctionne mécaniquement (le pipeline est correct), mais son effet réel sur la formation de chemins est masqué par le biais topologique à 50k, et inexistant aux grandes échelles avec les paramètres actuels.

---

## 4. Comportement des traces d'éligibilité

### Configuration

- `decay = 0.85` → demi-vie ≈ 4.3 ticks
- `max = 5.0` (clamp)
- `read_delay = 10` ticks (délai entre stimulus et readout/reward)

### Observations

| Métrique | 50k | 5M |
|---|---|---|
| Max éligibilité | 0.032 | Non mesuré |
| Non-nulles | 276 958 / 577 488 (48%) | 599 066 / 57M (~1%) |
| Diffusion | Large mais faible | Très sparse |

### Analyse

La trace d'éligibilité remplit son rôle de pont temporel STDP → reward, mais avec `decay = 0.85`, elle perd 85% de sa valeur en 10 ticks (le `read_delay`). Au moment du reward, l'éligibilité résiduelle est ~0.2^10 ≈ 0.001× la valeur initiale. C'est insuffisant pour un apprentissage robuste. Un `decay` plus lent (0.95-0.98) alignerait mieux la persistance avec le délai reward.

---

## 5. Qualité des entrées

### Mécanisme

`InputEncoder` injecte l'activation sur 50 nœuds sélectionnés spatialement (via KD-tree) pendant 5 ticks avec `intensity = 1.5`.

### Observations

- La sélection spatiale fonctionne correctement : centroïdes aux extrémités du domaine
- L'activité se propage sur ~2000-48000 nœuds selon la phase du cycle
- Le problème : l'injection est brève (5 ticks) et `activation_decay = 0.25` étouffe l'activité en ~8 ticks
- Le ratio input-nodes/total-nodes décroît avec la taille : 50/50k = 0.1%, 50/5M = 0.001%

### Verdict

Le mécanisme d'entrée est fonctionnel mais insuffisant pour les grandes tailles. Le `group_size` devrait probablement scaler avec √n ou log(n).

---

## 6. Qualité du readout de sortie

### Mécanisme

`OutputReader` somme les activations de 50 nœuds par classe, 10 ticks après la fin de l'injection. La classe avec le score le plus élevé est la décision.

### Observations à 50k

Le readout fonctionne parfaitement : 161/161 correct. Mais les scores bruts montrent un biais géométrique massif (classe correcte ≈ 300× vs classe incorrecte ≈ 0.5).

### Observations à 5M

Les scores sont `[0.5, 0.5]` — signal identique aux deux classes, cohérent avec le fait que l'activité ne se propage pas assez loin pour différencier les sorties (domaine 100× plus grand en volume, même 50 nœuds d'entrée).

---

## 7. Gains par rapport à V3

| Dimension | V3 | V4 |
|---|---|---|
| **Performance** | ~30 ms/tick CPU (50k) | **0.3 ms/tick GPU (50k)** — **100× plus rapide** |
| **Scaling** | Max ~50k nœuds | **5M nœuds testés** (57M arêtes) |
| **Mémoire** | ~45 GB pour 5M (crash OOM) | **4.0 GB stable** (après compaction) |
| **Architecture** | CPU-only, toutes les phases séquentielles | GPU-First, 11 shaders, 1 submit/tick |
| **Plasticité** | Hebbienne simple | Règle des 3 facteurs (STDP × éligibilité × dopamine) |
| **Apprentissage** | Pas de reward | Reward + dopamine spatialisée |
| **Consolidation** | Seuil fixe | Gatée par dopamine |
| **Backend** | CPU seul | `auto`/`cpu`/`gpu` via wgpu (Vulkan/Metal/DX12) |
| **Dispatch GPU** | — | 2D dispatch (supporte >65535 workgroups) |
| **I/O** | Fixe | Sélection spatiale KD-tree + trials automatiques |

---

## 8. Instabilités résiduelles

Voir [V4_instabilites.md](V4_instabilites.md) pour le détail complet. Résumé :

| Instabilité | Sévérité | Statut V4 |
|---|---|---|
| Accuracy 100% = biais topologique | **Critique** | Non résolu — la topologie fait le travail, pas l'apprentissage |
| Pas de preuve de chemins dopaminergiques | **Critique** | Non résolu — conductance ~+0.9% vs baseline |
| Éligibilité trop courte vs read_delay | Sévère | Non résolu — decay 0.85 incompatible avec delay 10 |
| Dynamique impulsionnelle (flash & die) | Majeur | Non résolu — activation_decay 0.25 trop agressif |
| PID en régime non-stationnaire | Modéré | Non résolu |
| Pas d'apprentissage à 500k/5M | **Nouveau** | Paramètres calibrés 50k ne scalent pas |

---

## 9. Décision de passage à V5

### Ce que la V4 a accompli

La V4 a **résolu le problème d'infrastructure** :
- Architecture GPU-First fonctionnelle et scalable (50k → 5M nœuds)
- Pipeline de plasticité complet (STDP → éligibilité → 3-facteurs → consolidation)
- Performance : 100× plus rapide que V3
- Mémoire : de 45 GB (crash) à 4 GB stable pour 5M nœuds

### Ce que la V4 n'a pas résolu

La V4 **n'a pas prouvé l'apprentissage** :
- L'accuracy 100% à 50k est un artefact du biais topologique
- À 500k/5M, accuracy = 50% (chance level)
- Les conductances ne montrent pas de chemins directionnels
- L'éligibilité décroît trop vite pour ponter le délai reward

### Recommandation : passage à V5

**Go pour V5**, avec focus sur la **preuve d'apprentissage** :

1. **Test anti-biais** : I/O à distances égales, ou association inversée (input-0 → output-1)
2. **Calibration multi-échelle** : hyperparamètres adaptatifs selon la taille du réseau
3. **Éligibilité plus lente** : decay 0.95-0.98 pour couvrir le read_delay
4. **Activité soutenue** : mécanismes de récurrence ou decay adaptatif
5. **Diagnostic de chemins** : outils de visualisation des routes de conductance renforcée

La V4 fournit le **moteur de calcul** ; la V5 doit fournir la **preuve de concept biologique**.
