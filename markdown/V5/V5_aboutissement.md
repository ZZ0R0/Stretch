# Aboutissement V5 (V5.0 → V5.2.1)

## 1. Résumé

La V5 a transformé l'infrastructure V4 en un système dont l'apprentissage est **scientifiquement démontré** et dont la **plasticité est constructive**.

Résultats fondamentaux :
- **V5.0** : Δ = +84 pp en mode inversé (CPU) — preuve d'apprentissage réel contre la géométrie
- **V5.2.1** : Δ = +5.1 pp en mode normal (GPU) — la plasticité améliore pour la première fois l'accuracy au lieu de la dégrader

La V5 a livré : le framework anti-biais topologique, le RPE (Reward Prediction Error), le pipeline GPU complet (15 phases), la dynamique soutenue, la visualisation 3D, et un corpus de 11+ expériences d'isolation systématique.

---

## 2. Chronologie des versions

| Version | Date | Apport principal | Résultat clé |
|---------|------|-----------------|--------------|
| **V5.0** | — | Preuve d'apprentissage (anti-biais, baselines, diagnostics) | Δ = +84 pp inversé (CPU) |
| **V5.0** | — | Dynamique soutenue (decay adaptatif, réverbération, reset) | +12.9 pp vs vanilla |
| **V5.1** | — | Visualisation complète (stretch-viz V5) | Carte 3D, chemins, sparklines |
| **V5.1** | — | RandomBaseline mesuré, Remap testé | 68.8%, 50% (trop lent) |
| **V5.2** | — | Orchestrateur unifié, RPE, modulation marge | Signal centré, protection plastique |
| **V5.2** | — | Port GPU V5 sustained (3 shaders, 15 phases) | Pipeline GPU complet |
| **V5.2** | — | Oubli accéléré (ρ_boost) | Homéostatique modulable par RPE |
| **V5.2.1** | — | Fix snapshot timing GPU | +13.3 pp, plasticité constructive |

---

## 3. Bilan des objectifs (vs cahier des charges)

### §3.1 — Tâches anti-biais topologique

| Exigence | Statut | Détail |
|----------|--------|--------|
| A. Symétrique | ✅ | I/O placés en croix autour du centre, distances égales |
| B. Association inversée | ✅ | `invert_mapping` + Symmetric : force le re-routing contre la géométrie |
| C. Re-apprentissage | ✅ | Remap (Legacy, inversion à tick 5000) : implémenté et testé |

### §3.2 — Baselines obligatoires

| Baseline | Config | Accuracy | Interprétation |
|----------|--------|----------|----------------|
| **RandomBaseline** | Symmetric, poids aléatoires 0.1–5.0 | **68.8%** | Structure accidentelle des poids aléatoires |
| **TopologyOnly** | Symmetric, plasticité off | **97.7%** (CPU normal) / **2.3%** (CPU inversé) | Biais topologique pur |
| **FullLearning** | Symmetric, plasticité on | **86.7%** (GPU normal) / **44.5%** (GPU inversé) | Apprentissage actif (post V5.2.1) |

### §3.3 — Calibration multi-échelle

| Statut | Détail |
|--------|--------|
| ⚠️ Partiel | Module `calibration.rs` implémenté (lois adaptatives gain/eligibility/decay vs N). Appliqué à 50k. Non testé à 500k/5M. |

### §3.4 — Dynamique soutenue

| Mécanisme | CPU | GPU | Paramètres |
|-----------|-----|-----|------------|
| Decay adaptatif | ✅ V5.0 | ✅ V5.2 | `k_local=0.3` |
| Réverbération locale | ✅ V5.0 | ✅ V5.2 | `reverb_gain=0.12` |
| Reset policy (full/partial/none) | ✅ V5.0 | ✅ V5.2 | `reset_policy="partial"` |
| Snapshot activations | ✅ V5.0 | ✅ V5.2 (fixé V5.2.1) | Post-reverb, pré-decay |
| Présentation configurable | ✅ V5.0 | ✅ V5.2 | `presentation_ticks=8` |

### §3.5 — RPE et signal de plasticité (V5.2)

| Composant | Statut | Détail |
|-----------|--------|--------|
| Baseline EMA $\bar{V}(t)$ | ✅ | $\alpha = 0.05$, convergence en ~20 ticks |
| RPE $\delta = r_{\text{eff}} - \bar{V}$ | ✅ | Signal centré en régime stationnaire |
| Modulation marge | ✅ | $r_{\text{eff}} = r / (1 + 0.1 \cdot |M|)$ |
| Oubli accéléré | ✅ | $\rho_{\text{eff}} = \rho_0 + \rho_b \cdot \max(0, -\delta)$ |
| Métriques exportées | ✅ | $\bar{V}$ et $\delta$ dans TickMetrics |

### §3.6 — Outils diagnostics

| Outil | Statut | Implémentation |
|-------|--------|----------------|
| Carte de conductance 3D | ✅ | `stretch-viz` mode Conductance (touche 4) |
| Path tracer input→output | ✅ | Dijkstra `log(C_max/C)`, overlay (touche P) |
| Timeline conductance | ✅ | Sparkline dans sidebar viz |
| Clusters co-renforcement | ✅ | Nœuds sur ≥2 chemins tracés (jaune dans viz) |
| Comparaison baseline vs learning | ✅ | Tableaux + superposition visuelle |

### §3.7 — Pipeline GPU complet (V5.2)

| Phase | Shader | Statut |
|-------|--------|--------|
| 0. Reset activations | `injection.wgsl` | ✅ (reset policy configurable) |
| 1. Inject stimulus | `injection.wgsl` | ✅ |
| 2. Zone measure | `zones.wgsl` | ✅ |
| 3. Zone PID | `zones.wgsl` | ✅ |
| 4. Zone apply | `zones.wgsl` | ✅ |
| 5. Source contribs | `source_contribs.wgsl` | ✅ |
| 6. Propagation | `propagation.wgsl` | ✅ |
| 7. Apply + dissipate | `apply_and_dissipate.wgsl` | ✅ |
| 7b. Reverberation | `reverberation.wgsl` | ✅ **NOUVEAU V5.2** |
| 7b'. Snapshot | `snapshot_activations.wgsl` | ✅ **NOUVEAU V5.2** (repositionné V5.2.1) |
| 7c. Adaptive decay | `adaptive_decay.wgsl` | ✅ **NOUVEAU V5.2** |
| 8. Plasticity | `plasticity.wgsl` | ✅ (RPE + ρ_boost V5.2) |
| 9. Budget sum | `budget.wgsl` | ✅ |
| 10. Budget scale + sync | `budget.wgsl` | ✅ |
| 11. Readout | `readout.wgsl` | ✅ (num_classes paramétrique V5.2) |

15 phases, single submit, ~9ms/tick (50k nœuds).

---

## 4. Résultats détaillés

### 4.1 Matrice historique (Symmetric, 50k nœuds, 10k ticks, CPU)

| | TopologyOnly | FullLearning | Δ |
|---|---|---|---|
| **Normal** (CPU V5.0) | 97.7% | 87.9% | −9.8 pp ❌ |
| **Inversé** (CPU V5.0) | 2.3% | **86.3%** | **+84.0 pp** ✅ |

### 4.2 Matrice finale (Symmetric, 50k nœuds, 10k ticks, GPU post V5.2.1)

| | TopologyOnly | FullLearning | Δ |
|---|---|---|---|
| **Normal** (GPU V5.2.1) | 81.6% | **86.7%** | **+5.1 pp** ✅ |
| **Inversé** (GPU V5.2.1) | 18.0% | **44.5%** | **+26.5 pp** ✅ |

**Lectures clés** :
- En mode normal, la plasticité **améliore** l'accuracy pour la première fois (+5.1 pp)
- En mode inversé, l'apprentissage remonte de 18% → 44.5% (+26.5 pp)
- Les deux Δ sont **positifs** — objectif fondamental de la V5.2 atteint

### 4.3 Évolution du Δ normal (FL vs Topo) à travers les versions

| Version | Backend | Δ Normal | Plasticité |
|---------|---------|----------|------------|
| V5.0 | CPU | -14.7 pp | Destructrice ❌ |
| V5.0 + sustained | CPU | -9.8 pp | Destructrice ❌ |
| V5.2 pre-fix | GPU | -3.2 pp | Quasi-neutre |
| **V5.2.1 post-fix** | **GPU** | **+5.1 pp** | **Constructive** ✅ |

### 4.4 Tableau complet des 13 expériences d'isolation

| # | Config | Backend | Sustained | Mode | Accuracy | Notes |
|---|--------|---------|-----------|------|----------|-------|
| 1 | Vanilla | CPU | OFF | TopologyOnly | **84.4%** | Baseline CPU pure |
| 2 | Vanilla | GPU | OFF | TopologyOnly | **82.0%** | GPU ≈ CPU (Δ=2.4pp) |
| 3 | Vanilla | CPU | OFF | FullLearning | **64.1%** | Plasticité : -20.3pp |
| 4 | V5 sustained | CPU | ON | TopologyOnly | **97.3%** | Sustained : +12.9pp |
| 5 | V5 sustained | CPU | ON | FullLearning | **87.9%** | Plasticité : -9.4pp |
| 6 | V5.2 pre-fix | GPU | ON | TopologyOnly | 76.6% | Bug snapshot |
| 7 | V5.2 pre-fix | GPU | ON | FullLearning | 73.4% | Bug snapshot |
| 8 | V5.2 no RPE | GPU | ON | FullLearning | 40.2% | Sans RPE : catastrophe |
| 9 | V5.2 Remap | GPU | ON | FullLearning | 25.0% | Consolidation bloque |
| 10 | **V5.2.1** | **GPU** | ON | TopologyOnly | **81.6%** | Fix : +5.0pp |
| 11 | **V5.2.1** | **GPU** | ON | FullLearning | **86.7%** | Fix : +13.3pp |
| 12 | V5.2.1 Inv | GPU | ON | FullLearning | 44.5% | Δ = +26.5 pp |
| 13 | V5.2.1 Inv | GPU | ON | TopologyOnly | 18.0% | Baseline inversée |

### 4.5 Remap (re-apprentissage)

50.0% global (V5.1, Legacy, remap tick 5000). 25.0% avec V5.2 GPU (consolidation bloque).

Le re-learning est le problème résiduel le plus critique : les edges consolidés pendant la phase initiale sont **immunisés** contre toute modification. Même l'oubli accéléré ($\rho_{\text{boost}} = 0.02$) ne peut pas les affecter.

---

## 5. Bugs corrigés pendant la V5

### V5.0
1. **Dijkstra OOM** : `-log(C)` donnait des coûts négatifs quand C > 1 → heap infinie → crash 32 Go. Fix : `log(5.0/C)`.
2. **invert_mapping non propagé** : `generate_trials()` recevait `placement.target_mapping` au lieu de `effective_mapping`. Fix : variable intermédiaire `effective_mapping`.
3. **VS Code rollback** : 3 régressions silencieusement rétablies (Dijkstra, config, mapping). Re-appliquées manuellement.
4. **.gitignore** : retour à la ligne manquant entre deux entrées.

### V5.2.1
5. **Snapshot timing GPU** : le snapshot des activations était en Phase 12 (post-decay) au lieu d'entre réverbération et adaptive_decay. L'énergie de réverbération GPU était ~75% du CPU. Fix : déplacement entre Phase 7b et 7c dans `run_full_tick()` et `run_profiled_tick()`. Impact : **+13.3 pp**.

Détails V5.0 dans `markdown/V5/V0/correction.md`.
Détails V5.2 dans `markdown/V5/V2/V5_maths_update.md` §4.

---

## 6. Visualisation (stretch-viz)

Refondue pour V5 avec :

| Fonctionnalité | Touche | Description |
|----------------|--------|-------------|
| Groupes I/O | I | Marqueurs colorés : In-0 (cyan), In-1 (magenta), Out-0 (vert), Out-1 (orange) |
| Chemins Dijkstra | P | Overlay des meilleurs chemins tracés entre I/O (colorés par classe) |
| Arêtes top conductance | C | 500 arêtes les plus modifiées, colorées par conductance |
| Mode Conductance | 4 | Coloration des nœuds par conductance sortante moyenne |
| Clusters | (auto) | Nœuds partagés par ≥2 chemins affichés en jaune |
| Accuracy sparkline | (auto) | Courbe d'accuracy dans la sidebar |
| Energy sparkline | (auto) | Courbe d'énergie dans la sidebar |
| Timeline conductance | (auto) | Évolution de la conductance moyenne |
| Refresh diagnostics | T | Recalcul des chemins et arêtes top |
| Infos V5 | (auto) | Task mode, baseline, mapping, RPE dans la sidebar |

---

## 7. Instabilités résiduelles

### 7.1 Plasticité CPU encore destructrice

Sur CPU, la plasticité dégrade toujours l'accuracy (-20.3 pp vanilla, -9.4 pp sustained). Causes identifiées :
- Champ dopaminergique trop diffus ($\lambda = 0.15$ sur $[0, 100]^3$ → rayon ~6.7 unités)
- Gain plastique trop élevé ($\eta = 3.0$)
- Homéostatique non sélective ($\rho \cdot (C_0 - C)$ efface les patterns utiles)

### 7.2 Gap GPU ↔ CPU (~10pp en TopologyOnly)

Le shader fusionné `apply_and_dissipate` met à jour fatigue/inhibition/trace AVANT la réverbération (GPU), alors que le CPU le fait APRÈS. Impact ~5pp. Fix : séparer le shader (V5.3).

### 7.3 Remap bloqué par consolidation

Consolidation irréversible empêche toute adaptation post-remap. Nécessite un mécanisme de dé-consolidation (V5.3).

### 7.4 Scaling non validé

Aucun test au-delà de 50k nœuds. Les lois de `calibration.rs` sont implémentées mais non vérifiées empiriquement.

---

## 8. Livrables

| Livrable | Statut | Version |
|----------|--------|---------|
| Moteur V5 (task, calibration, sustained, diagnostics) | ✅ | V5.0 |
| Visualisation V5 (stretch-viz complet) | ✅ | V5.1 |
| Orchestrateur unifié (process_readout, logique métier unique) | ✅ | V5.2 |
| RPE + modulation marge + oubli accéléré | ✅ | V5.2 |
| 3 shaders GPU (reverberation, adaptive_decay, snapshot) | ✅ | V5.2 |
| Pipeline GPU 15 phases (single submit) | ✅ | V5.2 |
| Fix snapshot timing | ✅ | V5.2.1 |
| 8 configs V5.2 + 7 configs V5.0 | ✅ | V5.0–V5.2 |
| 10 tests unitaires + 2 tests intégration | ✅ | V5.2 |
| Audit mathématique (V5_maths_update.md) | ✅ | V5.2 |
| 13 expériences d'isolation documentées | ✅ | V5.2 |
| Calibration multi-échelle | ⚠️ Implémentée, non testée >50k | V5.0 |
| Build : 0 erreurs, 0 warnings, 12/12 tests | ✅ | V5.2.1 |

---

## 9. Critères d'acceptation

| # | Critère | Verdict | Justification |
|---|---------|---------|---------------|
| 1 | Tâche anti-biais apprise | ✅ | 86.3% en inversé CPU (Δ +84 pp), 44.5% en inversé GPU (Δ +26.5 pp) |
| 2 | Baseline topology-only battue | ✅ | En inversé : 86.3% vs 2.3% (CPU), 44.5% vs 18.0% (GPU) |
| 3 | Plasticité constructive en mode normal | ✅ | GPU V5.2.1 : 86.7% vs 81.6% (Δ = +5.1 pp) |
| 4 | RPE centré en régime stationnaire | ✅ | $\mathbb{E}[\delta] \approx 0$ vérifié |
| 5 | Pipeline GPU V5 complet | ✅ | 15 phases, 14 shaders, 3 nouveaux sustained |
| 6 | Activité plus soutenue qu'en V4 | ✅ | Decay adaptatif + réverbération + reset partiel |
| 7 | Routes renforcées cohérentes | ⚠️ | D_k positifs en inversé ; en normal partiellement |
| 8 | Remap fonctionnel | ❌ | 25% — consolidation irréversible bloque |
| 9 | GPU ≈ CPU (±3pp) | ❌ | Écart ~10pp (shader fusionné) |
| 10 | Calibration >50k | ❌ | Non testé |
| 11 | Protocole archivé et reproductible | ✅ | Configs, seed=42, résultats documentés |

**Verdict global : V5 validée.**

Les critères centraux sont remplis :
- Preuve d'apprentissage irréfutable (Δ +84 pp inversé)
- Plasticité constructive en mode normal (+5.1 pp GPU)
- Pipeline GPU complet et fonctionnel

Les items non atteints (remap, gap GPU/CPU, scaling) sont identifiés avec causes et plans de correction pour V5.3.

---

## 10. Leçons retenues

1. **Le timing du snapshot est critique** : un décalage d'une phase dans un pipeline GPU de 15 passes peut coûter 13pp d'accuracy.

2. **Le RPE est nécessaire mais insuffisant** : il empêche la catastrophe plastique mais ne dirige pas positivement l'apprentissage.

3. **La consolidation est un couteau à double tranchant** : elle protège les bonnes routes mais empêche toute adaptation future.

4. **Les tests d'isolation systématique sont essentiels** : 13 expériences ciblées ont identifié la cause racine là où le symptôme pointait vers mille hypothèses.

5. **Le shader fusionné est une dette technique** : combiner trop de logique dans un seul shader crée des divergences de timing impossibles à corriger sans séparation.

6. **L'apprentissage est plus facile quand la tâche est difficile** : en inversé (topologie hostile), le signal d'erreur est informatif. En normal (topologie déjà correcte), le signal est saturé. Le RPE résout partiellement ce paradoxe.

---

## 11. Prochaines étapes — V6

La V6 ne vise pas de nouvelles features mais un **nettoyage profond** et un **rapprochement du comportement biologique**.

### Axe 1 — Suppression totale du code mort et des forks

Le code accumule des branches conditionnelles qui compliquent la maintenance et créent des divergences :
- `step_gpu()` / `step_cpu()` (~180 lignes chacun)
- RPE enabled/disabled, margin modulation on/off, spatial dopamine on/off
- Adaptive decay on/off, reverberation on/off, reset policy (3 modes)
- PID direct/indirect, calibration enabled/disabled
- Shader fusionné `apply_and_dissipate.wgsl` vs CPU split

**Principe V6** : pas de feature conditionnelle, tout doit être unique. Tout ce qui a prouvé son utilité en V5 (RPE, sustained, adaptive decay, réverbération) est **toujours activé**. Le code mort et les features non testées (calibration) sont supprimés. Le shader fusionné est séparé pour éliminer le gap GPU ↔ CPU.

### Axe 2 — Cohérence de la visualisation (chemins réels)

Le traceur Dijkstra (touche P) montre le chemin de coût minimal sur le graphe de conductances statique : $\text{cost}(e) = \ln(C_{\max}/C_e)$. C'est une **hypothèse géométrique**, pas une observation du signal réel. Un edge à haute conductance peut ne jamais être utilisé si son neurone source ne tire pas.

**Objectif V6** : visualiser les **chemins générés par l'apprentissage** — les routes que le signal emprunte réellement pendant un trial. Implémenter un traceur qui enregistre les neurones actifs et les arêtes ayant transmis du signal (pré ET post actifs) à chaque tick.

### Axe 3 — Évaluation rigoureuse de l'accuracy

L'accuracy actuelle ($\hat{c} = \arg\max_c \sum_{i \in \text{output}_c} a_i$, fenêtre 256 trials) prouve que le bon groupe de sortie domine, mais ne mesure ni la confiance ni la progression. Le Δ(FullLearning − TopologyOnly) est la seule preuve d'apprentissage réel.

**Objectif V6** : marge moyenne comme métrique primaire, courbes d'apprentissage tick-by-tick, séparabilité des distributions de scores, tests statistiques multi-seed (Welch's t-test).

### Axe 4 — Stabilité énergétique et réalisme biologique

L'énergie se propage en ondes massives ($G_{\text{eff}} = G \cdot \langle k \rangle \cdot \langle C \rangle \approx 1.44 > 1$) qui activent une majorité de neurones simultanément. Les défenses (fatigue, dissipation, PID) agissent après le passage de l'onde, pas pendant.

**Objectif V6** : rendre le comportement énergétique plus stable et plus proche d'un vrai réseau de neurones — inhibition latérale intra-tick, activation sparse (top-k par zone), $G_{\text{eff}} < 1$.

---

## Documentation associée

| Document | Contenu |
|----------|---------|
| `V5/V0/correction.md` | Bugs V5.0 (Dijkstra OOM, invert_mapping, rollback) |
| `V5/V0/maths.md` | Modélisation mathématique V5.0 |
| `V5/V0/vision_post_V5.0.md` | Diagnostic post-V5.0, axes V5.1 |
| `V5/V1/corrections.md` | Issues V5.1 (viz, gitignore, baselines) |
| `V5/V1/maths.md` | Maths V5.1 (projection, métriques, scoring) |
| `V5/V1/vision_post_v5.1.md` | Diagnostic post-V5.1, plan V5.2 |
| `V5/V2/V5_plan_realisation_global.md` | Plan maître V5.orch + V5.2 |
| `V5/V2/V5.orch_*.md` | 5 docs orchestrateur (CdC, archi, maths, protos, risques) |
| `V5/V2/V5.2_*.md` | 7 docs V5.2 (CdC, archi, maths, état des arts, protos, risques, aboutissement) |
| `V5/V2/V5_maths_update.md` | Audit mathématique complet + 13 expériences |
| `V5/V2/V5_vision_post_v5.md` | Correctifs & améliorations post-V5.2 (5 priorités) |
| `V5/V2/V5.2_math_conclusion.md` | Conclusion mathématique V5.2 |
| `V5/V2/V5.2_vision_post_V5.md` | Vision stratégique post-V5 |
| `V5/V3/V3_removable_code_forks.md` | Forks de code supprimables pour V5.3 |
| `V5/V5_vision_post_v5.md` | Synthèse globale post-V5 |
