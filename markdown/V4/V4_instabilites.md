# V4 — Instabilités et Limitations Identifiées

## 1. Dynamique impulsionnelle — « flash & die »

Le problème central de la V4 est que le système fonctionne **par impulsions**, pas par activité soutenue.

### Mécanisme observé

1. **Injection** (5 ticks) : 50 nœuds reçoivent intensity = 1.5 → activation grimpe
2. **Propagation** : l'activité se diffuse via KNN sur ~2000–7000 nœuds
3. **Effondrement** : `activation_decay = 0.25` (75% de perte par tick) → demi-vie ≈ 2.3 ticks
4. **Silence** : après 8 ticks, l'activité résiduelle est < 1% du pic
5. **Reset** : au tick suivant du trial, toutes les activations sont remises à zéro

### Conséquence

L'énergie du système oscille de manière **saccadée** entre ~700 et ~79 000, avec un ratio de 100x entre les pics (injection active) et les creux (inter-trial). Ce n'est pas une oscillation naturelle du réseau — c'est le reflet mécanique du cycle d'injection/extinction.

```
Énergie observée (échantillonnage à 100 ticks) :
t=0    →    500      (warm-up)
t=100  → 22 884      (pic mid-trial)
t=300  →    991      (creux inter-trial)
t=700  → 67 179      (pic)
t=800  →  3 079      (effondrement)
...
t=3800 → 79 242      (pic maximal)
t=3900 →  3 776      (effondrement)
```

Le graphe d'énergie montre un signal en **dents de scie** sans enveloppe stable, pas les sinusoïdes qu'un réseau auto-organisé devrait produire.

---

## 2. Absence de preuve de chemins dopaminergiques

### Ce qu'on mesure

| Métrique | Valeur (t=5000) | Interprétation |
|---|---|---|
| Conductance moyenne | 1.006 | Quasi-identique au baseline (1.0) |
| Arêtes > 1.05 | 65 710 / 577 488 (11.4%) | Renforcées faiblement |
| Arêtes < 0.95 | 68 622 / 577 488 (11.9%) | Affaiblies faiblement |
| Conductance max | 5.0 (clamp) | Quelques arêtes saturées |
| Éligibilité max | 0.032 | Très faible |
| Éligibilités non-nulles | 272 198 / 577 488 (47%) | Large mais diffuses |

### Analyse

La conductance moyenne n'a bougé que de **0.6%** par rapport au baseline après 5000 ticks et 161 trials. Même si ~23% des arêtes sont modifiées (au-delà de ±5%), la modification est **faible et symétrique** (autant renforcées qu'affaiblies). Cela ne constitue pas la preuve d'un chemin directionnel input → output.

### Pourquoi les chemins ne se forment pas clairement

1. **Décroissance homéostatique permanente** : `decay = 0.0001` ramène chaque arête vers `conductance = 1.0` à chaque tick, même pendant l'apprentissage
2. **Signal dopaminergique faible** : Δw = `plasticity_gain × δ_d × eligibility` = 2.0 × ~0.5 × ~0.03 = **~0.03 par événement reward**, mais l'homéostasie érode 0.0001 × (C-1.0) par tick en continu
3. **Éligibilité trop rapide** : `decay = 0.85` → demi-vie ≈ 4.3 ticks. L'éligibilité s'effondre avant que le reward (10 ticks après présentation) ne puisse l'exploiter significativement
4. **Activité trop brève** : STDP ne peut construire des ψ robustes si les nœuds ne sont actifs que 2-3 ticks

### Ce qu'on ne peut pas prouver

- Que la dopamine spatialisée crée des **routes préférentielles** de input-0 vers output-0
- Que le renforcement de conductance est **topologiquement cohérent** (pas juste aléatoire)
- Que le système pourrait apprendre une tâche plus complexe que la discrimination binaire

---

## 3. La question de l'accuracy à 100%

### Fait observé

161/161 trials corrects, 100% d'accuracy soutenue sur 5000 ticks.

### Problème : biais topologique

La configuration spatiale est **fortement biaisée** par construction :
- input-0 → centroïde (2.7, 50.6, 50.7) → **23 unités** de output-0 (24.9, 49.5, 49.5)
- input-0 → centroïde (2.7, 50.6, 50.7) → **73 unités** de output-1 (75.6, 49.7, 50.0)
- Ratio de distance : **3.2x**

Avec un kernel exponentiel (`exp(-0.3 × distance)`) :
- K(23) = exp(-6.9) ≈ 0.001
- K(73) = exp(-21.9) ≈ 3×10⁻¹⁰

Le signal arrive à output-0 avec un facteur **3 millions de fois** plus fort qu'à output-1, par la seule géométrie. Aucun apprentissage n'est nécessaire pour discriminer — la **topologie fait tout le travail**.

### Test manquant

Pour prouver que le système apprend réellement, il faudrait :
- Placer les I/O à distances égales des deux outputs
- Ou inverser les associations (input-0 → output-1) et vérifier que le réseau apprend à re-router

---

## 4. Instabilité du PID

### Configuration

- `target_activity = 0.3`
- Activité réelle : oscille entre 0 (inter-trial) et 0.14 (pic des 7000 actifs / 50 000)
- Le PID n'atteint jamais sa cible car l'activité est dominée par le cycle impulsionnel

### Impact

Le PID régule un signal qui n'a pas de stationnarité. Il essaie de stabiliser une grandeur qui varie de 100x à chaque cycle de trial — c'est un problème de mauvais régime, pas un bug de paramètres.

---

## 5. Résumé des instabilités

| Instabilité | Sévérité | Cause racine |
|---|---|---|
| Énergie saccadée (dents de scie) | **Critique** | activation_decay=0.25 + cycle d'injection 31 ticks |
| Pas de chemins dopaminergiques prouvés | **Critique** | Signal dopamine trop faible vs homéostasie |
| Accuracy 100% = biais topologique | **Critique** | Distance 3:1 rend la discrimination triviale |
| Éligibilité trop courte | Sévère | decay=0.85 → demi-vie 4.3 ticks < read_delay de 10 |
| PID en régime non-stationnaire | Modéré | Pas de point de fonctionnement stable |
| Activité pulsée, pas soutenue | Majeur | Decay agressif + reset explicite + pas de récurrence |
