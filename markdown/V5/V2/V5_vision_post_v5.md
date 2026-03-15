# Vision Post-V5.2 : Correctifs & Améliorations

> Basé sur l'audit mathématique et les résultats expérimentaux de V5_maths_update.md.
> Priorisé par impact attendu sur l'accuracy.

---

## Priorité 1 : Fix GPU Snapshot Timing (✅ IMPLÉMENTÉ — V5.2.1)

### Problème
Le GPU snapshote les activations **après** la dissipation (Phase 12), alors que
le CPU le fait **avant** (entre réverbération et dissipation). Résultat :
- CPU + sustained : 97.3% → GPU + sustained : 76.6% (TopologyOnly, Δ = **-20.7pp**)

### Solution (APPLIQUÉE)
Dans `gpu.rs::run_full_tick()`, snapshot déplacé de Phase 12 à entre Phase 7b
(reverberation) et Phase 7c (adaptive_decay). Même correction dans `run_profiled_tick()`.

### Résultats Mesurés
| Config | Avant Fix | Après Fix | Δ |
|--------|-----------|-----------|---|
| V5.2 Normal FullLearning (GPU) | 73.4% | **86.7%** | **+13.3pp** |
| V5.2 Normal TopologyOnly (GPU) | 76.6% | **81.6%** | **+5.0pp** |
| V5.2 Inverted FullLearning (GPU) | 55.5% | 44.5% | -11.0pp |
| V5.2 Inverted TopologyOnly (GPU) | 26.6% | 18.0% | -8.6pp |

**Plasticity désormais constructive** : FullLearning > TopologyOnly (+5.1pp).
Gap résiduel GPU↔CPU : ~10pp (fatigue/trace timing dans shader fusionné).

---

## Priorité 2 : Rendre la Plasticité Constructive (V5.3)

### Problème
La plasticité **dégrade** systématiquement l'accuracy :
| Mode | TopologyOnly | FullLearning | Δ |
|------|-------------|-------------|---|
| CPU vanilla | 84.4% | 64.1% | **-20.3pp** |
| CPU sustained | 97.3% | 87.9% | **-9.4pp** |

La règle des trois facteurs ne renforce pas sélectivement les bonnes connexions.

### Causes Identifiées

1. **Champ dopaminergique trop diffus** : $\lambda = 0.15$ sur un domaine [0,100]³ donne
   un rayon d'influence $r_{1/e} = 1/\lambda \approx 6.7$ unités. Avec un domaine de
   100 unités, ~80% des nœuds reçoivent un signal dopaminergique quasi-identique.
   Pas de sélectivité spatiale réelle.

2. **Gain plastique trop élevé** : $\eta = 3.0$ amplifie le bruit. Un signal
   dopaminergique diffus + un gain élevé = modifications aléatoires des conductances.

3. **Homéostatique comme force de rappel** : $\rho \cdot (C_0 - C)$ tire tout vers
   la baseline $C_0 = 1.0$, effaçant les patterns utiles.

### Propositions Mathématiques

#### 2a. Reward-gated plasticity (seuillage dopaminergique)
Ne modifier les conductances que si le signal récompense dépasse un seuil :

$$\Delta C = \begin{cases} \eta \cdot \delta_d \cdot e & \text{si } |\delta_d| > \delta_{\min} \\ 0 & \text{sinon} \end{cases}$$

avec $\delta_{\min} \approx 0.05$. Cela élimine les modifications sous bruit résiduel.

#### 2b. Spatial lambda plus fort
Augmenter $\lambda$ de 0.15 à 1.0–2.0 pour concentrer le signal dopaminergique
sur un rayon de 0.5–1.0 unité autour du centre de récompense. Seuls les nœuds
proches du readout sont modifiés.

$$\delta_{d,j} = D_{\text{phasic}} \cdot \exp(-\lambda_{\text{new}} \cdot \|p_j - p_{\text{reward}}\|)$$

#### 2c. Sign-specific plasticity
Distinguer le renforcement (reward) de la punition :
- Reward positif : ne renforcer que les edges dont $e > 0$ (potentialisation)
- Reward négatif : ne déprimer que les edges dont $e > 0$ (dépression des mêmes paths)

$$\Delta C = \begin{cases} \eta \cdot |\delta_d| \cdot e & \text{si } \delta_d > 0 \text{ et } e > 0 \\ -\eta \cdot |\delta_d| \cdot e & \text{si } \delta_d < 0 \text{ et } e > 0 \\ 0 & \text{sinon} \end{cases}$$

#### 2d. Learning rate scheduling
Réduire $\eta$ au fil du temps pour stabiliser les conductances :

$$\eta(t) = \eta_0 \cdot \max\!\Big(\frac{1}{1 + t/\tau_\eta},\; \eta_{\min}\Big)$$

avec $\tau_\eta = 3000$ ticks, $\eta_{\min} = 0.3$.

### Expériences de Validation Proposées
1. Vanilla CPU FullLearning avec $\delta_{\min} = 0.05$ → mesurer Δ vs TopologyOnly
2. Vanilla CPU FullLearning avec $\lambda = 1.5$ → mesurer Δ
3. Grid search : ($\lambda$, $\eta$) sur $\{0.5, 1.0, 2.0\} \times \{0.5, 1.0, 3.0\}$
4. Comparer accuracy curves (convergence speed + final accuracy)

---

## Priorité 3 : Mécanisme de Dé-Consolidation (V5.3)

### Problème
Le remap est complètement bloqué (25%) car les edges consolidés sont permanents.

### Proposition : Consolidation avec durée de vie

$$\text{consolidated}_{ij}(t) = \begin{cases}
\text{true} & \text{si conditions remplies} \\
\text{false} & \text{si } t - t_{\text{consol}} > T_{\text{decons}} \text{ et } D_{\text{phasic}} < D_{\text{thresh}}
\end{cases}$$

Un edge consolidé se dé-consolide si :
1. Le signal dopaminergique reste faible pendant $T_{\text{decons}}$ ticks
2. Son éligibilité est négative (le path n'est plus utile)

Paramètres proposés :
- $T_{\text{decons}} = 500$ ticks (~13 essais sans récompense)
- Condition : $\bar{D}_{\text{phasic,local}} < 0.05$ ET $\bar{e}_{ij} < -0.1$

### Impact Attendu
Remap accuracy : 25% → >50% (permet l'oubli structurel)

---

## Priorité 4 : RPE Constructif (V5.3)

### Problème
Le RPE actuel n'influence que $\rho_{\text{eff}}$ (oubli accéléré quand négatif).
Il ne guide pas positivement l'apprentissage.

### Proposition : RPE comme modulateur de gain plastique

$$\eta_{\text{eff}} = \eta_0 \cdot \Big(1 + \alpha_{\text{RPE-gain}} \cdot \text{clamp}(\delta_{\text{RPE}},\; -1,\; 1)\Big)$$

- RPE positif (performance améliorée) → $\eta_{\text{eff}} > \eta_0$ → apprentissage accéléré
- RPE négatif (performance dégradée) → $\eta_{\text{eff}} < \eta_0$ → apprentissage ralenti
- RPE ≈ 0 (performance stable) → $\eta_{\text{eff}} \approx \eta_0$

Avec $\alpha_{\text{RPE-gain}} = 0.5$ :
- $\delta_{\text{RPE}} = +1$ → $\eta_{\text{eff}} = 1.5 \cdot \eta_0$ (boost 50%)
- $\delta_{\text{RPE}} = -1$ → $\eta_{\text{eff}} = 0.5 \cdot \eta_0$ (frein 50%)

---

## Priorité 5 : Optimisations de Performance (V5.4)

### 5a. Multi-seed averaging
Actuellement, chaque config est testée avec 1 seed. Pour des résultats fiables :
- CLI `--seeds 5 config.toml` → 5 runs, report mean ± std
- Automatiser les comparaisons A/B avec tests statistiques

### 5b. Diagnostic en temps réel
Ajouter des métriques per-tick :
- Énergie de réverbération : $E_{\text{rev}} = \sum_i |a_i^{\text{snap}}|$
- Ratio conductances modifiées / total
- Distribution de $\delta_d$ sur les hot edges
- Zone-per-zone accuracy breakdown

### 5c. Hyperparameter search automatisé
Script de grid search parallèle sur les 4 axes critiques :
1. $\lambda_{\text{spatial}}$ ∈ {0.15, 0.5, 1.0, 2.0}
2. $\eta$ ∈ {0.5, 1.0, 2.0, 3.0}
3. $\rho$ ∈ {0.00001, 0.0001, 0.001}
4. $\delta_{\min}$ ∈ {0.0, 0.02, 0.05, 0.1}

= 192 configurations × 3 seeds = 576 runs. Avec 9ms/tick GPU :
~5 minutes total sur GPU moderne.

---

## Feuille de Route

| Phase | Contenu | Effort | Impact |
|-------|---------|--------|--------|
| **V5.2.1** | Fix snapshot GPU | 1 ligne dans `gpu.rs` | +20pp GPU |
| **V5.3a** | Reward-gated plasticity | ~20 lignes | Plasticity constructive |
| **V5.3b** | Dé-consolidation | ~30 lignes | Remap débloqué |
| **V5.3c** | RPE constructif | ~10 lignes | Signal d'apprentissage dirigé |
| **V5.3d** | λ et η calibration | Config only | Quick wins |
| **V5.4** | Multi-seed + grid search | Script CLI | Rigueur expérimentale |

### Critères de Succès
1. **GPU ≈ CPU** : Écart < 3pp sur TopologyOnly (post fix snapshot)
2. **Plasticity constructive** : FullLearning > TopologyOnly (au moins +5pp)
3. **Remap fonctionnel** : Accuracy > 60% après remap
4. **Reproductibilité** : Résultats stables sur 5 seeds (σ < 3pp)
