# Vision Post-V5 — Synthèse globale

> Ce document consolide la vision stratégique après l'ensemble de la V5 (V5.0, V5.1, V5.2).
> Il synthétise les acquis, les problèmes résiduels, et les axes d'évolution.

---

## 1. Chronologie V5

| Version | Apport principal | Résultat clé |
|---------|-----------------|--------------|
| **V5.0** | Preuve d'apprentissage (tâches anti-biais, baselines) | Δ = +84 pp en inversé (CPU) |
| **V5.0** | Dynamique soutenue (decay adaptatif, réverbération, reset) | +12.9 pp vs vanilla (CPU) |
| **V5.0** | Diagnostics (Dijkstra, RouteScore, D_k, CT) | Chemins reconfigurés visibles |
| **V5.1** | Visualisation complète (stretch-viz V5) | Carte 3D, chemins, sparklines |
| **V5.1** | RandomBaseline mesuré | 68.8% (borne inférieure structurelle) |
| **V5.1** | Remap testé | 50% (trop lent, 5000 ticks insuffisants) |
| **V5.2** | Orchestrateur unifié (process_readout) | Élimination duplication CPU/GPU |
| **V5.2** | RPE + modulation marge | Signal centré, protection plastique |
| **V5.2** | Port GPU V5 sustained (3 shaders, 15 phases) | Pipeline GPU complet |
| **V5.2** | Oubli accéléré (ρ_boost) | Homéostatique modulable par RPE |
| **V5.2.1** | Fix snapshot timing GPU | +13.3 pp, plasticité constructive |

---

## 2. État actuel du système

### 2.1 Architecture

```
stretch-core (~7920 lignes)
├── 17 modules Rust (simulation, gpu, stdp, config, domain, ...)
├── 14 shaders WGSL (injection, zones, propagation, plasticity, ...)
└── Pipeline GPU : 15 phases, single submit, ~9ms/tick (50k nœuds)

stretch-viz (~620 lignes)
└── Visualisation 3D, carte conductance, chemins Dijkstra, sparklines

stretch-cli (~210 lignes)
└── CLI : config TOML, --backend, --seeds
```

### 2.2 Modèle mathématique

12 composants vérifiés cohérents entre équations, code CPU, et code GPU :

1. Propagation CSR avec kernel exponentiel
2. Seuil effectif (fatigue, inhibition, zones PID)
3. Dissipation standard (jitter) + adaptative V5
4. Réverbération locale V5
5. STDP ($A^\pm = 0.008$, $\tau^\pm = 20$)
6. Éligibilité ($\gamma = 0.95$)
7. Règle des trois facteurs ($\eta = 3.0$, $\lambda = 0.15$)
8. Homéostatique ($\rho = 0.0001$) + oubli accéléré V5.2
9. Consolidation (seuil + durée + dopa)
10. Budget synaptique ($B = 30$)
11. RPE ($\delta = r_{\text{eff}} - \bar{V}$, $\alpha = 0.05$)
12. Modulation marge ($\beta = 0.1$)

### 2.3 Résultats les plus récents (V5.2.1, GPU)

| | TopologyOnly | FullLearning | Δ |
|---|---|---|---|
| Normal | 81.6% | **86.7%** | **+5.1 pp** ✅ |
| Inversé | 18.0% | **44.5%** | **+26.5 pp** ✅ |

Comparaison historique (Normal, FullLearning) :
- V5.0 CPU : 83.0% (Δ = -14.7 pp) ❌
- V5.1 CPU : 87.9% (Δ = -9.8 pp) ❌
- **V5.2 GPU : 86.7% (Δ = +5.1 pp) ✅**

---

## 3. Problèmes non résolus

### 3.1 Plasticité destructrice sur CPU

| Backend | TopologyOnly | FullLearning | Δ |
|---------|-------------|-------------|---|
| CPU vanilla | 84.4% | 64.1% | -20.3 pp ❌ |
| CPU sustained | 97.3% | 87.9% | -9.4 pp ❌ |
| GPU sustained | 81.6% | 86.7% | +5.1 pp ✅ |

Sur GPU, la plasticité est constructive grâce à la combinaison RPE + fix snapshot. Sur CPU, elle reste destructrice. Causes :
- Champ dopaminergique trop diffus ($\lambda = 0.15$)
- Gain plastique trop élevé ($\eta = 3.0$)
- Pas de seuillage de $\delta_d$ (toute variation → modification)

### 3.2 Remap bloqué (25%)

Consolidation irréversible : les edges renforcés pendant la phase initiale sont immunisés contre toute modification post-remap. Même l'oubli accéléré ($\rho_{\text{boost}} = 0.02$) ne peut pas les affecter.

### 3.3 Gap GPU ↔ CPU (~10pp)

Le shader fusionné `apply_and_dissipate.wgsl` crée un décalage de timing :
- GPU : fatigue/inhibition/trace mis à jour AVANT la réverbération
- CPU : ces mises à jour se font APRÈS

Impact : ~5pp. Nécessite de séparer le shader.

### 3.4 Scaling non testé

Aucun benchmark au-delà de 50k nœuds. Les lois de calibration multi-échelle (`calibration.rs`) sont implémentées mais non validées.

---

## 4. Feuille de route

### V6 — Nettoyage, réalisme, rigueur

La V6 n'ajoute pas de features. Elle **nettoie**, **stabilise**, et **vérifie** ce que la V5 a construit.

#### Axe 1 : Suppression totale du code mort et des forks

| Action | Type | Justification |
|--------|------|---------------|
| Supprimer le chemin CPU dupliqué (`step_cpu`) | Refactoring | Un seul pipeline GPU, CPU = fallback minimal |
| Activer RPE, sustained, adaptive decay, reverb en permanence | Config cleanup | Tout ce qui a prouvé son utilité est toujours ON |
| Supprimer calibration non testée | Dead code | `calibration.rs` jamais validé >50k |
| Séparer le shader fusionné `apply_and_dissipate` | Architecture | Élimine le gap GPU ↔ CPU (~10pp) |
| Éliminer les feature toggles (margin_mod, spatial dopa, etc.) | Simplification | Pas de feature conditionnelle |

**Résultat** : un codebase où chaque ligne de code est exercée à chaque simulation.

#### Axe 2 : Visualisation des chemins réels d'apprentissage

| Action | Type | Justification |
|--------|------|---------------|
| Implémenter un traceur de signal réel (activation bitmap par tick) | Nouveau code | Voir les routes que le signal emprunte réellement |
| Enregistrer les arêtes actives (pré ET post actifs) par trial | Instrumentation | Identifier les synapses effectivement utilisées |
| Visualiser le flux effectif input → output | Viz | Remplacer l'hypothèse Dijkstra par l'observation |
| Comparer chemins réels vs Dijkstra | Diagnostic | Mesurer la pertinence de CT |

**Résultat** : la touche P montre les chemins **réels**, pas les chemins **hypothétiques**.

#### Axe 3 : Évaluation rigoureuse de l'apprentissage

| Action | Type | Justification |
|--------|------|---------------|
| Marge moyenne comme métrique primaire | Métrique | Plus informative que l'accuracy binaire |
| Courbes d'apprentissage tick-by-tick exportées | Export | Montrer la progression, pas juste le résultat final |
| Séparabilité des distributions $P(S_c \mid \text{target})$ vs $P(S_c \mid \text{non-target})$ | Statistique | Prouver que la discrimination s'améliore |
| Multi-seed systématique (5 seeds, $\mu \pm \sigma$) | Protocole | Prouver la reproductibilité |
| Tests statistiques A/B (Welch's t-test) | Statistique | Prouver la significativité du Δ |

**Résultat** : une réponse définitive à « le réseau apprend-il vraiment ? ».

#### Axe 4 : Stabilité énergétique et réalisme biologique

| Action | Type | Justification |
|--------|------|---------------|
| Inhibition latérale intra-tick | Nouveau mécanisme | Empêcher l'onde avant qu'elle se forme |
| Activation sparse (top-k ou softmax par zone) | Architecture | Limiter le % de neurones actifs simultanément |
| Réduire $G_{\text{eff}} < 1$ | Paramétrique | Chaque étape atténue au lieu d'amplifier |
| Métriques de sparsité (% nœuds actifs par tick) | Monitoring | Vérifier que l'activité reste sparse |
| Benchmarks biologiques (1-5% d'activation simultanée) | Objectif | Critère de réalisme mesurable |

**Résultat** : un réseau dont le comportement énergétique ressemble à un vrai réseau de neurones.

---

## 5. Métriques de gate V5 → V6

| Critère | Seuil | Atteint |
|---------|-------|---------|
| Plasticité constructive sur tous backends | Δ ≥ 0 | Partiel (GPU ✅, CPU ❌) |
| Inversé appris | Δ ≥ +20 pp | ✅ |
| Remap fonctionnel | > 60% | ❌ (25%) |
| GPU ≈ CPU | < 5pp écart | ❌ (~10pp) |
| Reproductibilité (σ < 3pp, 5 seeds) | Mesuré | Non testé |
| Build propre, tests verts | 100% | ✅ |

La V6 ne requiert pas que tous les gates soient atteints. Certains (remap, gap CPU/GPU) seront **résolus par** la V6 elle-même (suppression des forks → pipeline unique → gap éliminé).

---

## 6. Conclusion

La V5 a rempli sa mission première : **prouver l'apprentissage réel** au-delà du biais topologique. Le Δ de +84 pp (V5.0 inversé) est une preuve non ambiguë. La V5.2 a enrichi cette fondation avec le RPE, le pipeline GPU complet, et la première plasticité constructive en mode normal (+5.1 pp).

Mais quatre problèmes fondamentaux subsistent :

1. **Le code est devenu un labyrinthe de forks** — chaque feature a son toggle, chaque backend a son chemin, les divergences silencieuses coûtent des semaines de debugging.

2. **La visualisation montre des hypothèses, pas des faits** — le chemin Dijkstra est un calcul mathématique sur un graphe statique, pas une observation de ce que le réseau fait réellement.

3. **L'accuracy ne prouve pas l'apprentissage seule** — sans marge, sans courbe de progression, sans test statistique, le nombre 86.7% ne dit pas grand-chose. Seul le Δ est informatif, et encore : sa significativité n'est pas testée.

4. **L'énergie explose en raz de marée** — la propagation active la majorité des neurones simultanément, rendant l'éligibilité non-sélective et la plasticité aveugle.

La V6 s'attaque à ces quatre points. Pas de nouvelles features, pas de scaling, pas de hiérarchie. D'abord **nettoyer**, **observer correctement**, **mesurer rigoureusement**, et **stabiliser**. L'architecture et le modèle sont solides — il faut maintenant les épurer et les valider.
