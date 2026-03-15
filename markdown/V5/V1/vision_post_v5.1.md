# Vision post-V5.1 — Vers la V5.2

## Ce que la V5.1 laisse en héritage

### Acquis V5.0 (rappel)

- **Preuve d'apprentissage** : Δ = +84 pp en mode inversé (86.3% vs 2.3%)
- **Framework anti-biais** : géométrie symétrique, inversion de mapping, trois baselines
- **Diagnostics structurels** : RouteScore, D_k, CT, SustainRatio
- **Moteur complet** : task, calibration, sustained, diagnostics

### Acquis V5.1 (nouveaux)

1. **Visualisation V5 opérationnelle** : `stretch-viz` entièrement refondu avec carte I/O, chemins Dijkstra, arêtes top conductance, mode Conductance par nœud, clusters de co-renforcement, sparklines accuracy/energy/conductance, panel V5 info. Le cahier des charges §4.5 est rempli.

2. **RandomBaseline mesuré** : 68.8% en Symmetric. Ce chiffre constitue la **borne inférieure structurelle** — l'accuracy obtenue par la géométrie + poids aléatoires sans aucun apprentissage. Le FullLearning inversé (86.3%) est 17.5 pp au-dessus, confirmant l'effet de la plasticité.

3. **Remap testé** : 50.0% global (Legacy, inversion à tick 5000). Le réseau ré-apprend trop lentement — les routes renforcées en phase 1 persistent et dominent en phase 2. L'homéostasie ($\rho = 0.0001$) est insuffisante pour un re-routage rapide.

4. **Build propre** : 0 erreurs, 0 warnings sur les 3 crates. Toutes les lacunes documentaires et techniques identifiées en V5.0 sont comblées.

### Matrice de résultats complète (V5.0 + V5.1)

| Condition | TopologyOnly | RandomBaseline | FullLearning | Δ (FL vs Topo) |
|---|---|---|---|---|
| **Normal** [0→0, 1→1] | 97.7% | 68.8% | 87.9% | −9.8 pp ❌ |
| **Inversé** [0→1, 1→0] | 2.3% | — | 86.3% | +84.0 pp ✅ |
| **Remap** (invert@5000) | — | — | 50.0% | — |

---

## Questions ouvertes (héritées de V5.0, non résolues)

### 1. Reward saturation (inchangé, priorité absolue)

En mode normal : $p(r = +1) \approx 0.98$. La dopamine est quasi-constante. Le renforcement est en nappe (toutes les arêtes actives, bonnes ou mauvaises). Résultat : la plasticité **détruit** l'accuracy topologique naturelle (−9.8 pp).

Le diagnostic V5.0 reste valide : le problème n'est pas l'entropie du reward mais l'**asymétrie d'action** — renforcer tout ≠ affaiblir le mauvais.

### 2. Re-learning trop lent (nouveau, révélé par V5.1)

Le test Remap (50.0%) montre que le réseau ne peut pas ré-apprendre en 5000 ticks. L'homéostasie est trop lente :

$$\Delta C^{\text{homéo}} = \rho \cdot (C_0 - C_{ij}) = 0.0001 \cdot (1.0 - 3.0) = -0.0002 \text{ /tick}$$

Pour ramener une arête de $C = 3.0$ à $C = 1.0$ : ~10 000 ticks d'homéostasie pure. Avec la plasticité active qui re-renforce partiellement, c'est encore plus lent.

### 3. Dégradation vs topologie (inchangé)

L'apprentissage dégrade les performances en mode normal (87.9% vs 97.7%). La V5.1 n'a pas modifié le modèle de plasticité — le problème persiste.

### 4. Scaling non testé (inchangé)

Les lois de calibration multi-échelle sont implémentées dans `calibration.rs` mais n'ont été testées qu'à 50k nœuds.

---

## Diagnostic post-V5.1 : deux problèmes, une racine

La **saturation du reward** (question 1) et le **re-learning lent** (question 2) partagent la même racine : le signal d'apprentissage ne distingue pas les résultats **attendus** des résultats **surprenants**.

- En mode normal, 98% de succès → dopamine constamment positive → renforcement aveugle → dégradation.
- En mode remap post-inversion, anciennes routes donnent ~0% → dopamine constamment négative → mais l'affaiblissement uniforme est trop lent pour effacer 5000 ticks de renforcement.

La solution est la même dans les deux cas : **le reward prediction error (RPE)**.

---

## Axes prioritaires pour la V5.2

### Axe 1 : Reward baseline / RPE (priorité absolue)

**Problème** : la dopamine phasique est pilotée par $r(t)$ brut. Quand le réseau réussit systématiquement, $r \approx +1$ constant → pas de signal informatif.

**Solution** : soustraire une moyenne glissante :

$$\delta(t) = r(t) - \bar{r}(t)$$
$$\bar{r}(t + 1) = (1 - \alpha) \cdot \bar{r}(t) + \alpha \cdot r(t), \quad \alpha \in [0.01, 0.1]$$

Remplacer $r(t)$ par $\delta(t)$ dans le calcul de la dopamine phasique.

**Effets attendus** :

| Mode | $\bar{r}$ | Succès → $\delta$ | Erreur → $\delta$ | Effet |
|------|-----------|---------------------|---------------------|-------|
| Normal | ≈ +1.0 | ≈ 0 (pas de renforcement) | ≈ −1.5 (correction forte) | Protège la topologie |
| Inversé | ≈ −0.5 | ≈ +1.5 (renforcement fort) | ≈ 0 (pas de modification) | Renforce les nouveaux chemins |
| Remap post-inv. | transition | Signal fort pour les deux | Signal fort pour les deux | Re-routing accéléré |

**Critère de succès** :
- Normal : FullLearning ≥ TopologyOnly (Δ ≥ 0)
- Inversé : FullLearning ≥ 80% (Δ ≥ +77 pp)
- Remap : accuracy phase 2 > 70% en 5000 ticks

### Axe 2 : Oubli accéléré pour le remap

Le test Remap V5.1 a montré que $\rho = 0.0001$ est trop lent. Deux pistes :

**Piste A** — Augmenter $\rho$ dynamiquement quand le reward moyen chute :

$$\rho_{\text{eff}} = \rho_0 + \rho_{\text{boost}} \cdot \max(0, -\delta(t))$$

L'homéostasie s'accélère quand le réseau fait systématiquement des erreurs (post-remap).

**Piste B** — Decay multiplicatif des conductances sur les chemins échoués :

$$C_{ij} \leftarrow C_{ij} \cdot (1 - \beta), \quad \beta \in [0.001, 0.01]$$

appliqué aux arêtes dont les deux endpoints étaient actifs pendant un essai échoué.

**Critère de succès** : accuracy > 70% en phase 2 du Remap (5000 ticks post-inversion).

### Axe 3 : Modulation par la marge (hérité de V5.0)

Le readout fournit une marge $M = S_{\text{gagnant}} - S_{\text{perdant}}$. Pondérer le reward :

$$r_{\text{eff}} = r \cdot \frac{1}{1 + \beta_M \cdot |M|}$$

Quand la marge est grande, le reward est atténué (essai trivial → peu d'apprentissage). Quand la marge est petite, le reward est maximal (essai ambigu → apprentissage maximal).

**Critère de succès** : variance inter-exécutions réduite, convergence plus monotone.

### Axe 4 : Multi-seed et déterminisme (hérité de V5.0)

Le non-déterminisme rayon crée ±5 pp de variance. Pour des benchmarks fiables :

1. Option `num_threads = 1` pour forcer l'exécution séquentielle
2. Chaque test exécuté avec $N \geq 5$ seeds, résultats rapportés en $\mu \pm \sigma$
3. Export de la courbe d'accuracy tick par tick (déjà implémenté dans la sparkline)

### Axe 5 : Scaling (conditionné à l'axe 1)

Si le RPE résout la saturation : activer la calibration multi-échelle et tester à 200k, 500k, 1M nœuds. Les lois dans `calibration.rs` ajustent automatiquement les gains et decays en fonction de $N$.

Cet axe reste **bloqué** tant que l'apprentissage dégrade les performances en mode normal.

---

## Matrice 2×2 cible pour la V5.2

La V5.0+V5.1 a produit :

| | TopologyOnly | FullLearning | Δ |
|---|---|---|---|
| Normal [0,1] | 97.7% | 87.9% | **−9.8 pp** ❌ |
| Inversé [1,0] | 2.3% | 86.3% | **+84.0 pp** ✅ |

La V5.2 doit produire :

| | TopologyOnly | FullLearning | Δ |
|---|---|---|---|
| Normal [0,1] | ~97% | **≥ 97%** | **≥ 0 pp** ✅ |
| Inversé [1,0] | ~3% | **≥ 80%** | **≥ +77 pp** ✅ |

Plus un nouveau critère Remap :

| | Phase 1 (0–5k) | Phase 2 (5k–10k) |
|---|---|---|
| Remap | > 90% | **> 70%** |

**Gate V5.1 → V5.2** : les deux Δ doivent être ≥ 0 et le Remap phase 2 > 70%.

---

## Risques V5.2

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| RPE trop conservateur ($\delta \approx 0$ en régime → arrêt total de l'apprentissage) | Moyenne | Modéré | Tuner $\alpha$ (plus petit = mémoire longue = $\bar{r}$ lent → $\delta$ plus grand). Plancher $|\delta| \geq \epsilon$. |
| Oubli accéléré détruit les routes en mode normal (pas seulement en remap) | Faible | Élevé | Conditionner l'oubli accéléré au mode Remap ou à une chute détectée du reward moyen |
| Interaction RPE × marge × oubli imprévisible | Moyenne | Modéré | Implémenter un axe à la fois : 1 → 2 → 3. Ne jamais activer simultanément. |
| Le scaling révèle que les lois de calibration sont fausses à >200k | Moyenne | Modéré | Conditionné à l'axe 1. Si le RPE marche à 50k, tester 200k d'abord avant 1M. |

---

## Ordre d'implémentation recommandé

```
V5.2a : RPE seul (axe 1)
  → Tester matrice 2×2 normal + inversé
  → Si Δ_normal ≥ 0 → succès, passer à V5.2b
  → Sinon → tuner α, investiguer

V5.2b : RPE + oubli accéléré (axes 1+2)
  → Tester Remap
  → Si phase 2 > 70% → succès, passer à V5.2c
  → Sinon → piste B (decay multiplicatif)

V5.2c : RPE + oubli + marge (axes 1+2+3)
  → Multi-seed (axe 4)
  → Mesurer variance et convergence

V5.2d : Scaling (axe 5)
  → 200k puis 500k puis 1M
```

---

## Résumé

La V5.1 a **complété** la V5.0 : le visualiseur est opérationnel, les trois baselines sont mesurées, le test Remap est exécuté. La V5 dans son ensemble a prouvé l'apprentissage réel (Δ +84 pp) et identifié deux problèmes précis : la saturation du reward et la lenteur du re-learning.

Le diagnostic est clair : le réseau apprend quand c'est difficile mais dégrade quand c'est facile. La solution candidate — le RPE — est théoriquement fondée et largement validée en neurosciences computationnelles (Schultz 1997) et en RL (REINFORCE avec baseline).

La V5.2 doit transformer Stretch d'un système qui « apprend quand la topologie est hostile » en un système qui « ne détruit jamais ce qui marche et améliore ce qui ne marche pas ». C'est la transition de la preuve de concept vers la fiabilité algorithmique.
