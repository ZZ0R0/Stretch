# V4 — Instabilités et Limitations Identifiées

> **Document mis à jour** après les trois phases de corrections (V4.1 refactoring, V4.2 GPU compute, V4.3 GPU-First)
> et les tests de scaling à 500k et 5M nœuds.

---

## 1. Dynamique impulsionnelle — « flash & die »

**Sévérité : Majeure — Non résolu en V4**

Le système fonctionne **par impulsions**, pas par activité soutenue.

### Mécanisme observé

1. **Injection** (5 ticks) : 50 nœuds reçoivent intensity = 1.5 → activation grimpe
2. **Propagation** : l'activité se diffuse via KNN sur ~2000–48000 nœuds (selon la taille)
3. **Effondrement** : `activation_decay = 0.25` (75% de perte par tick) → demi-vie ≈ 2.3 ticks
4. **Silence** : après 8 ticks, l'activité résiduelle est < 1% du pic
5. **Reset** : au tick suivant du trial, toutes les activations sont remises à zéro

### Données mesurées (50k nœuds, GPU-First, 0.3 ms/tick)

```
Énergie observée (échantillonnage à 100 ticks) :
t=0    →       500   (warm-up)
t=100  →   25 303    (pic mid-trial)
t=300  →      974    (creux inter-trial)
t=700  →   67 179    (pic)
t=800  →    3 079    (effondrement)
t=3800 →   97 004    (pic maximal observé)
t=3900 →    3 731    (effondrement)
```

Le graphe d'énergie montre un signal en **dents de scie** sans enveloppe stable, pas les sinusoïdes qu'un réseau auto-organisé devrait produire.

### Aggravation à grande échelle (5M nœuds)

À 5M nœuds, le problème s'amplifie :
- **50 nœuds injectés / 5M total = 0.001%** — le stimulus est microscopique
- L'activité oscille entre 0 (creux) et 48 084 nœuds actifs soit < 1% du réseau
- L'énergie ne dépasse jamais 342 227 pour un maximum théorique bien plus élevé
- Le réseau n'atteint jamais un régime d'activité auto-entretenue

### Piste V5

Mécanismes de récurrence, decay adaptatif selon l'activité locale, ou injection proportionnelle à √n.

---

## 2. Absence de preuve de chemins dopaminergiques

**Sévérité : Critique — Non résolu en V4**

### Données mesurées (50k nœuds, 5000 ticks, 161 trials)

| Métrique | Valeur | Interprétation |
|---|---|---|
| Conductance moyenne | 1.009 | +0.9% du baseline (1.0) |
| Arêtes > 1.05 | 48 556 / 577 488 (8.4%) | Renforcées faiblement |
| Arêtes < 0.95 | 49 029 / 577 488 (8.5%) | Affaiblies faiblement |
| Conductance max | 5.0 (clamp) | Quelques arêtes saturées |
| Éligibilité max | 0.032 | Très faible |
| Éligibilités non-nulles | 276 958 / 577 488 (48%) | Large mais diffuses |
| Modified edges exported | 176 896 / 577 488 (31%) | Modification détectable sur ~1/3 |

### Données à 5M nœuds (1000 ticks, 32 trials)

| Métrique | Valeur | Interprétation |
|---|---|---|
| Conductance moyenne | **0.304** | Chute de 70% — le decay homéostatique domine |
| Conductance max | 1.763 | Faible renforcement même pour la meilleure arête |
| Arêtes > 1.05 | 3 661 / 57M (<0.01%) | Quasi inexistant |
| Modified edges | 222 507 / 57M (0.4%) | Signal d'apprentissage dilué dans la masse |

### Analyse

La modification de conductance est **quasi-symétrique** (autant renforcées qu'affaiblies) et **topologiquement incohérente** — il n'y a pas de preuve de routes directionnelles input → output.

À 5M nœuds, le phénomène empire dramatiquement : le decay homéostatique (`0.0001 × (C - 1.0)` par tick) agit sur **toutes** les 57M arêtes constamment, tandis que le signal dopaminergique ne touche que les quelques milliers d'arêtes proches des I/O.

### Causes identifiées

1. **Homéostasie trop forte vs signal d'apprentissage** : decay permanent ≈ 0.0001/tick × 1000 ticks = -10% cumulé, tandis que le renforcement par événement ≈ +0.03
2. **Signal dopaminergique trop faible** : `Δw = η × δ_d × e = 2.0 × ~0.5 × ~0.03 ≈ 0.03` par événement
3. **Éligibilité trop courte** (voir §4)
4. **Activité trop brève** pour construire des traces STDP robustes

---

## 3. Accuracy 100% = biais topologique

**Sévérité : Critique — Non résolu en V4**

### Fait observé

161/161 trials corrects à 50k nœuds, 100% d'accuracy soutenue.

### Biais géométrique

| Paire | Distance | K (kernel exp -0.3×d) | Ratio |
|---|---|---|---|
| input-0 → output-0 | 23.1 | 1.0×10⁻³ | **1×** (référence) |
| input-0 → output-1 | 73.2 | 3.2×10⁻¹⁰ | **3×10⁶** plus faible |
| input-1 → output-1 | 22.1 | 1.3×10⁻³ | ~1.3× |
| input-1 → output-0 | 72.2 | 4.3×10⁻¹⁰ | **3×10⁶** plus faible |

Le signal arrive à la sortie correcte avec un facteur **3 millions de fois** plus fort que la sortie incorrecte. La topologie fait tout le travail — aucun apprentissage n'est nécessaire.

### Preuve par les scores bruts

```
[GPU t=4995 trial=160] target=0 dec=0 scores=[306.82, 0.50]
```

Score classe correcte : 306.82 vs classe incorrecte : 0.50 — ratio 600:1.

### Contraste avec les grandes échelles

À 500k et 5M nœuds, l'accuracy tombe à **50% (chance level)** — les scores readout sont `[0.5, 0.5]`. Le signal ne se propage tout simplement pas assez loin pour différencier les sorties, même avec le biais topologique. Ceci confirme que le problème est double :
1. À 50k, la topologie résout tout (pas besoin d'apprendre)
2. À 5M, rien ne résout (ni topologie, ni apprentissage)

### Tests manquants pour V5

- I/O à distances **égales** des deux outputs
- Association **inversée** (input-0 → output-1) pour forcer un re-routage
- Métriques de **cohérence topologique** des conductances renforcées

---

## 4. Éligibilité trop courte

**Sévérité : Sévère — Non résolu en V4**

### Configuration

- `eligibility.decay = 0.85` → demi-vie ≈ 4.3 ticks
- `output.read_delay = 10` ticks (délai stimulus → reward)

### Problème

Au moment du reward (tick t + 10), l'éligibilité résiduelle vaut :
```
e(t+10) = e(t) × 0.85^10 = e(t) × 0.20
```

80% du signal STDP est perdu avant que la dopamine ne puisse l'exploiter. C'est un **design mismatch** entre la vitesse de décroissance et le délai reward.

### Impact

- Seules les arêtes activées dans les 2-3 derniers ticks avant le reward reçoivent un ΔC significatif
- Les arêtes activées au début de la propagation (celles proches de l'input, potentiellement les plus importantes) ont une éligibilité ~0 au moment du reward

### Recommandation V5

`eligibility.decay = 0.95` → demi-vie ≈ 14 ticks → rétention au reward : `0.95^10 = 0.60` (60% préservé au lieu de 20%).

---

## 5. Instabilité du PID

**Sévérité : Modérée — Non résolu en V4**

### Configuration

- `target_activity = 0.3`
- Activité réelle : oscille entre 0 (inter-trial) et 0.14 (pic des 7000 actifs / 50 000)
- Le PID n'atteint jamais sa cible car l'activité est dominée par le cycle impulsionnel

### Impact

Le PID régule un signal qui n'a pas de stationnarité. Il essaie de stabiliser une grandeur qui varie de 100× à chaque cycle de trial. Les corrections `threshold_mod` et `gain_mod` oscillent en permanence sans converger.

### Note à grande échelle

À 5M nœuds, le ratio actifs/total est encore plus faible (~0.01% → 0.96%), rendant le PID encore moins pertinent.

---

## 6. Scaling : paramètres non-adaptatifs

**Sévérité : Critique — Nouvelle instabilité identifiée en V4**

### Problème

Les hyperparamètres sont calibrés pour 50k nœuds et ne scalent **pas** :

| Paramètre | Valeur 50k | Effet à 5M |
|---|---|---|
| `group_size = 50` | 0.1% des nœuds | 0.001% — stimulus microscopique |
| `gain = 0.8` | Propagation sur ~7000 nœuds | Idem (~7000) — ne touche que 0.14% du réseau |
| `spatial_lambda = 0.15` | Dopamine locale suffisante | Dopamine locale = 0.001% des arêtes |
| `decay = 0.0001` | Érosion lente | 57M arêtes × 0.0001 = érosion massive cumulée |
| `target_activity = 0.3` | Jamais atteinte à 50k | Encore plus irréaliste à 5M |

### Données

| Échelle | Accuracy | Conductance moy. | Arêtes modifiées |
|---|---|---|---|
| 50k | 100% (biais topo) | 1.009 (+0.9%) | 31% |
| 500k | 50% (chance) | Non mesuré | Non mesuré |
| 5M | 50% (chance) | 0.304 (-70% !) | 0.4% |

### Recommandation V5

Paramètres adaptatifs : `group_size ∝ √n`, `gain ∝ log(n)`, `spatial_lambda ∝ extent / √n`, `decay ∝ 1/n`.

---

## 7. Performance et mémoire (résolu)

**Sévérité : Résolu en V4.3 + corrections scaling**

### Historique des corrections

| Problème | Version | Solution | Résultat |
|---|---|---|---|
| 30 ms/tick CPU (50k) | V4.2 | GPU compute via wgpu | ~5 ms/tick |
| 5 ms/tick hybride (50k) | V4.3 | GPU-First architecture | **0.3 ms/tick** |
| OOM à 5M (45 GB RSS) | Post-V4.3 | `compact_for_gpu()` + scoped uploads + `HashSet` | **4.0 GB stable** |
| Crash dispatch >65535 | Post-V4.3 | `dispatch_2d()` + shaders 2D | ✅ |
| Crash buffer limits | Post-V4.3 | `adapter.limits()` | ✅ |

### Performance finale mesurée

| Échelle | ms/tick | GPU_TICK ms | Metrics ms (overhead) |
|---|---|---|---|
| 50k nœuds | **0.3** | 0.2 | 0.15 |
| 5M nœuds | **24.6** | 4.7 | 17.3 (dominant) |

Note : à 5M, le temps est dominé par le calcul de métriques (qui nécessite un readback GPU périodique), pas par le compute GPU pur (~5 ms).

---

## 8. Résumé des instabilités

| # | Instabilité | Sévérité | Cause racine | Statut V4 |
|---|---|---|---|---|
| 1 | Dynamique impulsionnelle (flash & die) | Majeur | `activation_decay=0.25` + injection 5 ticks | ❌ Non résolu |
| 2 | Pas de chemins dopaminergiques prouvés | **Critique** | Signal dopamine trop faible vs homéostasie | ❌ Non résolu |
| 3 | Accuracy 100% = biais topologique | **Critique** | Distance 3:1 rend discrimination triviale | ❌ Non résolu |
| 4 | Éligibilité trop courte | Sévère | decay 0.85 → 80% perdu avant reward | ❌ Non résolu |
| 5 | PID en régime non-stationnaire | Modéré | Activité trop impulsionnelle | ❌ Non résolu |
| 6 | Paramètres non-adaptatifs (scaling) | **Critique** | 50k params → 50% accuracy à 500k/5M | ❌ Nouveau |
| 7 | Performance et mémoire | Résolu | GPU-First + compact_for_gpu | ✅ **Résolu** |
