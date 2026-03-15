# Vision post-V5.0 — Vers la V5.1

## Ce que la V5.0 laisse en héritage

### Acquis fondamentaux

La V5.0 a répondu à la question centrale posée par la V4 : **le réseau apprend-il réellement ?**

- **Preuve d'apprentissage irréfutable** : en configuration anti-biais (mapping inversé), accuracy topologique = 2.3%, accuracy avec apprentissage = 86.3%. Le Δ = **+84 pp** ne peut s'expliquer que par la plasticité.
- **Vérification de cohérence** : $A_{\text{topo,normal}} + A_{\text{topo,inversé}} = 97.7\% + 2.3\% = 100\%$. Les baselines sont le miroir exact l'une de l'autre.
- **Framework de test anti-biais** : système de tâches configurable (Symmetric, Legacy, Inverted, Remap) avec `invert_mapping`, baselines (TopologyOnly, RandomBaseline, FullLearning).
- **Diagnostics structurels** : RouteScore, indice directionnel $D_k$, cohérence topologique $CT$, sustain ratio.
- **Stabilité V5** : decay adaptatif, réverbération locale, politique de reset configurable.

### Questions ouvertes (non résolues)

1. **L'apprentissage dégrade les performances en mode normal** : $97.7\% \to 83\%$ = -14.7 pp. La plasticité ajoute du bruit quand la topologie est déjà optimale.
2. **Saturation du signal de reward** : quand 97.7% des essais donnent $r = +1$, la dopamine est quasi-constante. Le renforcement est en nappe (toutes les arêtes actives, bonnes ou mauvaises) plutôt que ciblé.
3. **Credit assignment absent** : la règle des trois facteurs $\Delta C_{ij} = \eta \cdot \delta_D \cdot \mathcal{E}_{ij}$ ne distingue pas les arêtes causalement responsables du succès de celles qui étaient simplement actives.
4. **Non-déterminisme rayon** : ±5 pp de variance entre exécutions identiques. Acceptable pour les preuves qualitatives, insuffisant pour des benchmarks fins.
5. **Scaling non testé** : la V5.0 n'a été validée qu'à 50k nœuds. Les lois de calibration multi-échelle sont implémentées mais désactivées.

---

## Diagnostic Post-V5.0

### Pourquoi la plasticité dégrade en mode normal

C'est le problème central à résoudre. Analyse quantitative :

En mode normal, $\sim 97.7\%$ des essais sont corrects par topologie → reward $r = +1$ → dopamine positive → $\Delta C > 0$ pour toutes les arêtes éligibles. Le budget synaptique ($B = 30$) force la redistribution : renforcer des arêtes inutiles affaiblit les arêtes qui portaient le signal topologique naturel.

En mode inversé, $\sim 97.7\%$ des essais sont _incorrects_ → reward $r = -0.5$ → dopamine négative → les mauvais chemins (favorisés par la topologie) sont activement affaiblis. Les rares succès créent un signal distinctif. **L'entropie du reward est maximale quand la tâche est difficile.**

Formellement, l'information portée par le reward :

$$H(r) = -\sum_r p(r) \log p(r)$$

- Mode normal : $p(+1) \approx 0.98$, $p(-0.5) \approx 0.02$ → $H \approx 0.14$ bits
- Mode inversé : $p(+1) \approx 0.02$, $p(-0.5) \approx 0.98$ → $H \approx 0.14$ bits

L'entropie est la même ! La différence est que **le signal négatif affaiblit activement les mauvais chemins** (ce qui est constructif) alors que **le signal positif renforce tout** (ce qui est destructeur quand la topologie est déjà bonne).

Le problème n'est donc pas l'entropie mais l'**asymétrie d'action** : renforcer tout ≠ affaiblir le mauvais.

---

## Axes prioritaires pour la V5.1

### Axe 1 : Reward baseline (priorité absolue)

**Problème** : la plasticité ne devrait se déclencher que sur les **surprises**, pas sur les résultats attendus.

**Solution** : soustraire une moyenne glissante du reward :

$$\delta(t) = r(t) - \bar{r}(t)$$

$$\bar{r}(t+1) = (1 - \alpha) \cdot \bar{r}(t) + \alpha \cdot r(t), \quad \alpha \in [0.01, 0.1]$$

Utiliser $\delta(t)$ au lieu de $r(t)$ pour driver la dopamine phasique.

**Effet attendu** :
- En mode normal : $\bar{r} \approx +1$, donc un succès donne $\delta \approx 0$ (pas de renforcement), une erreur donne $\delta \approx -1.5$ (signal fort pour corriger)
- En mode inversé : $\bar{r} \approx -0.5$, donc un succès donne $\delta \approx +1.5$ (signal fort pour renforcer), une erreur donne $\delta \approx 0$ (pas de modification)
- Le système apprend uniquement quand le résultat est **inattendu**

C'est l'équivalent direct du **REINFORCE avec baseline** en RL, ou du **reward prediction error** des neurones dopaminergiques biologiques (Schultz 1997).

**Critère de succès** : en mode normal, l'apprentissage doit **maintenir ou améliorer** l'accuracy topologique, pas la dégrader.

### Axe 2 : Modulation de la plasticité par la marge

**Problème** : le readout fournit une marge (score\_gagnant - score\_perdant), mais cette information est jetée.

**Solution** : pondérer la force du reward par l'inverse de la marge :

$$r_{\text{eff}} = r \cdot \frac{1}{1 + \beta \cdot |\text{margin}|}$$

Quand la marge est grande (décision facile), le reward est atténué. Quand la marge est petite (décision incertaine), le reward maximal s'applique. Le système concentre l'apprentissage sur les essais ambigus.

**Critère de succès** : variation de l'accuracy réduite entre exécutions, convergence plus monotone.

### Axe 3 : Credit assignment spatial amélioré

**Problème** : la dopamine spatialisée ($\lambda_D = 0.15$) décroît lentement — à 30 unités du centre, $e^{-4.5} \approx 1.1\%$ du signal persiste. C'est trop diffus pour un réseau de 50k nœuds dans un cube de 100.

**Pistes** :
1. **Augmenter $\lambda_D$** : 0.15 → 0.3+ pour concentrer la dopamine plus près du reward center
2. **Masque d'éligibilité par activité** : ne mettre à jour que les arêtes dont les _deux_ endpoints ont été actifs pendant la fenêtre de présentation du trial courant (pas d'un trial précédent)
3. **Dopamine à deux centres** : un centre positif (output correct) ET un centre négatif (output prédit si erreur), au même tick

**Critère de succès** : CT > 0.1 et D < 0 pour les deux classes en mode inversé.

### Axe 4 : Stabilité et robustesse

1. **Déterminisme** : option pour forcer l'exécution séquentielle (rayon::ThreadPoolBuilder avec 1 thread) pour les benchmarks reproductibles. Le coût est acceptable pour la validation.
2. **Multi-seed** : chaque test doit être exécuté avec N seeds (N ≥ 5) et les résultats moyennés avec écart-type rapport.
3. **Courbe d'apprentissage** : exporter l'accuracy glissante (fenêtre de 20 essais) tick par tick pour visualiser la dynamique.

### Axe 5 : Scaling (déferred)

Les lois de calibration multi-échelle (implémentées dans `calibration.rs`) doivent être activées et testées à 200k, 500k, 1M. Cet axe est conditionné au succès de l'axe 1 — aucun intérêt à scaler un système qui dégrade les performances.

---

## Matrice 2×2 cible pour la V5.1

La V5.0 a produit :

| | TopologyOnly | FullLearning | Δ |
|---|---|---|---|
| Normal [0,1] | 97.7% | 83% | **-14.7 pp** ❌ |
| Inversé [1,0] | 2.3% | 86.3% | **+84.0 pp** ✅ |

La V5.1 doit produire :

| | TopologyOnly | FullLearning | Δ |
|---|---|---|---|
| Normal [0,1] | ~97% | **≥ 97%** | **≥ 0 pp** ✅ |
| Inversé [1,0] | ~3% | **≥ 80%** | **≥ +77 pp** ✅ |

**Gate V5.0 → V5.1** : les deux Δ doivent être positifs ou nuls. L'apprentissage ne doit jamais dégrader.

---

## Risques V5.1

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| Reward baseline rend le système trop conservateur (ne modifie plus rien) | Moyenne | Modéré | Tuner $\alpha$ et vérifier que le Δ inversé reste > 70 pp |
| Modulation par la marge crée des oscillations (renforce puis affaiblit en boucle) | Faible | Modéré | Clipper $r_{\text{eff}}$ et monitorer la stabilité de la courbe d'apprentissage |
| Le credit assignment est fondamentalement impossible sans backprop | Faible | Critique | La preuve V5.0 (+84 pp) montre que le système CAN learn sans backprop. Le problème est la précision, pas la capacité. |
| Les corrections d'axes 1-3 interagissent de manière imprévisible | Moyenne | Modéré | Implémenter et tester un axe à la fois (1 → 2 → 3), pas tous ensemble |

---

## Résumé

La V5.0 a **prouvé** que Stretch est capable d'apprentissage réel (Δ +84 pp contre topologie hostile). C'est un résultat fondamental — aucune version précédente ne l'avait démontré.

Le problème identifié est précis et caractérisé : **le reward saturé détruit les performances quand la topologie est favorable**. La correction candidate (reward baseline / RPE) est théoriquement fondée (Schultz 1997, REINFORCE avec baseline).

La V5.1 doit transformer Stretch d'un système qui « sait apprendre quand c'est dur » en un système qui « ne détruit pas ce qui marche et améliore ce qui ne marche pas ». C'est le passage de la preuve de concept à la fiabilité.