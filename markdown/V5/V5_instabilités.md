# Instabilités V5 — Analyse complète

> Ce document recense toutes les instabilités identifiées au cours de la V5 (V5.0 → V5.2.1),
> qu'elles soient corrigées ou résiduelles. Il constitue le diagnostic de référence pour la V6.

---

## 1. Explosions d'énergie — Ondes de type « raz de marée »

### 1.1 Symptôme

Lors de la présentation d'un stimulus, l'activation se propage sous forme d'**onde massive** qui active la quasi-totalité des neurones du réseau. Le pattern d'activation n'est pas sélectif : au lieu d'un chemin sparse entre les nœuds d'entrée et de sortie, c'est une **vague omnidirectionnelle** qui submerge le réseau entier.

Ce comportement est biologiquement irréaliste. Dans un vrai réseau de neurones, seule une fraction sparse des neurones s'active simultanément (~1-5%). Ici, les pics d'énergie peuvent toucher >50% des nœuds.

### 1.2 Mécanisme

La propagation suit :

$$S_i = a_i \cdot \text{sign}_i \cdot (1 + g_{\text{mod},i}) \cdot G$$

Avec $G = 0.15$ (gain propagation) et une connectivité moyenne $\langle k \rangle \approx 12$ (k-NN), le **gain effectif** par étape est :

$$G_{\text{eff}} = G \cdot \langle k \rangle \cdot \langle C \rangle \approx 0.15 \times 12 \times 0.8 \approx 1.44$$

Comme $G_{\text{eff}} > 1$, chaque étape de propagation **amplifie** le signal au lieu de l'atténuer. Même si la fatigue et la dissipation finissent par éteindre la vague après quelques ticks, la couverture instantanée est déjà massive.

### 1.3 Mécanismes de défense existants

| Couche | Mécanisme | Force | Limitation |
|--------|-----------|-------|------------|
| L1 : Fatigue | $f_i$ élève le seuil exponentiellement | Forte | Délai : agit APRÈS le premier pic |
| L2 : Inhibition | Second modificateur de seuil, décroissance lente | Forte | Délai similaire |
| L3 : Dissipation | 35%/tick de décroissance exponentielle | Forte | N'empêche pas le front d'onde initial |
| L4 : Budget synaptique | $\sum C_{\text{out}} \leq B = 30$ | Limite dure | Contraint le steady-state, pas le transitoire |
| L5 : PID zones | Modulation des seuils par zone | Adaptatif | **Trop lent** (~50 ticks de latence) |
| L6 : Decay adaptatif | $\alpha_i \propto (1 - k_{\text{local}} \cdot \text{activité locale})$ | Conditionnel | Actif uniquement V5 sustained |
| L7 : Clamping | Activation bornée à $[0.01, 10.0]$ | Absolu | Empêche l'explosion numérique, pas l'onde |

**Diagnostic** : les défenses agissent **après** le passage de l'onde. Aucun mécanisme ne **prévient** la formation de l'onde. Il manque :
- Une **inhibition latérale rapide** (dans le même tick)
- Un **seuillage d'activation sparse** (ex : winner-take-all par zone)
- Un **contrôle du fan-out effectif** en temps réel

### 1.4 Impact sur l'apprentissage

Quand tous les neurones s'activent simultanément :
- La trace d'éligibilité est non-sélective → la plasticité renforce tout
- Le signal reward/punition ne peut pas discriminer les « bons » vs « mauvais » chemins
- Le RPE perd sa valeur informative car $\bar{V}$ reflète l'énergie totale, pas la qualité du routing

### 1.5 Direction V6

Objectif : **rendre le comportement énergétique plus stable et plus proche d'un vrai réseau de neurones**. Pistes :
- Inhibition latérale intra-tick (les neurones inhibiteurs agissent dans le même pas de propagation)
- Activation competitive (top-k ou softmax par zone au lieu de seuillage individuel)
- Réduction du gain effectif ($G_{\text{eff}} < 1$) par ajustement de $G$ ou du fan-out

---

## 2. Exactitude de l'évaluation de l'accuracy

### 2.1 Méthode actuelle

L'accuracy est calculée par :

1. **Score readout** par classe : $S_c = \sum_{i \in \text{output}_c} a_i$
2. **Décision** : $\hat{c} = \arg\max_c S_c$
3. **Comptage** : `correct_count / total_evaluated` sur les $N$ derniers trials (fenêtre glissante, $N = 256$)

Le score est une simple somme d'activations des nœuds de sortie, sans aucune transformation (pas de softmax, pas de normalisation).

### 2.2 Limites fondamentales

**Question centrale : cette accuracy prouve-t-elle que le réseau apprend vraiment ?**

#### A. Biais topologique dominant

En mode Normal/Symmetric, TopologyOnly obtient **81.6%** (GPU) voire **97.7%** (CPU). La géométrie seule (distances I→O) suffit à produire un classement correct dans la majorité des cas. L'accuracy absolue ne distingue pas :
- « Le réseau a appris » de « La géométrie favorise déjà la bonne réponse »

**Seul le Δ(FullLearning − TopologyOnly)** apporte une preuve d'apprentissage réel :
- Δ = +5.1 pp en Normal GPU → la plasticité aide modestement
- Δ = +26.5 pp en Inversé GPU → la plasticité reconstruit réellement des routes
- Δ = +84 pp en Inversé CPU → preuve maximale

#### B. Absence de métriques de confiance

La décision par argmax ne mesure pas la **certitude** de la décision :
- Un écart de 0.001 entre les deux scores produit la même décision correcte qu'un écart de 5.0
- La **marge** ($M = S_{\text{best}} - S_{\text{second}}$) existe dans le code (`ReadoutResult.margin`) mais n'est pas utilisée dans le calcul d'accuracy
- Aucune mesure de **séparabilité** des distributions de scores par classe

#### C. Contamination par les ondes d'énergie

Si l'onde tsunami active uniformément tous les neurones, les nœuds de sortie des deux classes reçoivent une énergie similaire. La décision repose alors sur des **micro-différences** d'activation liées à la géométrie (distance au stimulus), pas à l'apprentissage.

#### D. Fenêtre glissante sans historique

La fenêtre de 256 trials rapporte l'accuracy **récente** mais ne montre pas :
- La **courbe d'apprentissage** (progression vs. stagnation)
- Les épisodes de **régression** (accuracy qui baisse puis remonte)
- La **variance** inter-essais

### 2.3 Ce que l'accuracy actuelle prouve réellement

| Ce qu'elle prouve | Ce qu'elle ne prouve pas |
|-------------------|--------------------------|
| Le groupe de sortie correct a plus d'énergie | Que le routing est sélectif |
| Le Δ montre un effet de la plasticité | Que l'apprentissage est robuste |
| Le mode inversé force un vrai re-routing | Que la confiance augmente avec le temps |
| Le RPE améliore vs. dégradation | Que les chemins learned sont stables |

### 2.4 Améliorations V6

- **Marge moyenne** comme métrique primaire (pas seulement accuracy binaire)
- **Courbe d'apprentissage** tick-by-tick exportée
- **Séparabilité** : distance entre distributions $P(S_c | c = \text{target})$ et $P(S_c | c \neq \text{target})$
- **Tests statistiques** : Welch's t-test sur multi-seed pour prouver la significativité du Δ
- **Taux de confiance** : proportion de décisions avec marge > seuil

---

## 3. Déconnexion Dijkstra — Chemins réels d'apprentissage

### 3.1 Ce que montre le P-key (Dijkstra)

Le traceur Dijkstra implémenté dans `diagnostics.rs` calcule le chemin de coût minimal sur le graphe de conductance :

$$\text{cost}(e) = \ln\left(\frac{C_{\max}}{C_e}\right)$$

C'est un **algorithme de graphe statique** appliqué post-hoc sur la matrice de conductances. Il répond à la question : *« Quel est le chemin le plus conducteur entre les nœuds d'entrée et de sortie ? »*

### 3.2 Ce qu'il ne montre PAS

| Dijkstra | Signal réel |
|----------|-------------|
| Snapshot instantané des conductances | Dynamique temporelle tick-par-tick |
| Un seul chemin optimal | Multiples chemins parallèles (redondance) |
| Toutes les arêtes considérées | Seules les arêtes dont le pré-neurone a tiré |
| Aucune information temporelle | Séquence d'activation ordonnée |
| Déterministe | Stochastique (STDP, dopamine, éligibilité) |

**Le problème fondamental** : le chemin Dijkstra est une **hypothèse géométrique** sur le routing, pas une observation du routing réel. Un edge à conductance élevée peut ne jamais être utilisé si son neurone source ne s'active pas lors du trial.

### 3.3 Ce que l'utilisateur voudrait voir

Des **chemins générés par l'apprentissage** — les routes que le signal emprunte réellement pendant un trial :

1. **Trace d'activation temporelle** : quels neurones s'activent à chaque tick après le stimulus
2. **Edges actifs** : quels synapses ont effectivement transmis du signal (pré ET post actifs)
3. **Convergence vers la sortie** : le flux effectif du stimulus vers les nœuds de sortie
4. **Évolution** : comment ces chemins changent au cours de l'entraînement

### 3.4 La cohérence topologique (CT) comme pont partiel

Le diagnostic `topological_coherence` mesure la corrélation entre Δ conductance et appartenance au chemin Dijkstra :

$$CT = \text{Pearson}(\Delta C_e, \mathbb{1}_{e \in \text{path}})$$

- CT > 0.5 → la plasticité a renforcé les chemins que Dijkstra identifie → convergence
- CT ≈ 0 → aucune relation → Dijkstra non prédictif
- CT < 0 → la plasticité a affaibli les chemins Dijkstra → divergence

CT est utile mais reste indirect : il mesure si la plasticité est **alignée** avec l'hypothèse géométrique, pas si le réseau utilise réellement ces chemins.

### 3.5 Direction V6

Implémenter un **traceur de signal réel** qui enregistre, pour chaque trial :
- Les neurones actifs à chaque tick (bitmap d'activation)
- Les arêtes ayant transmis du signal (pré actif ET post actif)
- Le flux effectif (contribution de chaque arête au score readout)

Puis visualiser ces chemins réels au lieu des chemins Dijkstra hypothétiques.

---

## 4. Prolifération des forks de code

### 4.1 Inventaire des forks

Le code contient de nombreux chemins conditionnels qui compliquent la maintenance et créent des divergences de comportement :

| Fork | Localisation | Type | Impact |
|------|-------------|------|--------|
| `step_gpu()` / `step_cpu()` | `simulation.rs:678-679` | Dispatch backend | ~180 lignes dupliquées chacun |
| TopologyOnly / FullLearning / RandomBaseline | `simulation.rs:418-422` | Baseline mode | Plasticité on/off |
| `backend = "cpu"` / `"gpu"` / `"auto"` | `config.rs:597-606` | Config | 3 chemins d'init |
| RPE enabled/disabled | `reward.rs:34` | Feature toggle | Deux chemins reward |
| Margin modulation on/off | `reward.rs:38` | Feature toggle | Deux chemins reward |
| Spatial dopamine on/off | `simulation.rs:649-660` | Feature toggle | Champ dopa conditionnel |
| Adaptive decay on/off | `simulation.rs:948-1000` | V5 sustained | Deux dissipations |
| Reverberation on/off | `simulation.rs:928-937` | V5 sustained | Optionnel |
| Reset policy full/partial/none | `sustained.rs:85-106` | V5 sustained | 3 chemins |
| PID direct/indirect | `zone.rs:147` | V3 mode | 2 algorithmes PID |
| Calibration enabled/disabled | `calibration.rs` | V5 calibration | Module entier conditionnel |
| 4 types de topologie | `domain.rs:95-115` | Config | 4 constructeurs |
| Shader fusionné vs CPU split | `apply_and_dissipate.wgsl` | Architecture | Gap ~5pp CPU↔GPU |

### 4.2 Conséquences

1. **Divergence GPU ↔ CPU** : le shader fusionné crée un écart de ~10pp car l'ordre des opérations diffère
2. **Surface de test exponentiell** : 3 baselines × 3 backends × 2 RPE × 2 margins × 3 resets × 2 PID = 216 combinaisons théoriques, testées pour < 15
3. **Maintenance quadratique** : chaque nouvelle feature doit être implémentée sur tous les chemins
4. **Bugs silencieux** : une feature peut fonctionner sur GPU mais pas sur CPU (cas du snapshot timing, découvert après des semaines)

### 4.3 Direction V6

**Suppression totale du code mort et des forks** :
- Pas de feature conditionnelle : tout doit être unique ou presque
- Un seul backend cible (GPU, le CPU n'est plus qu'un fallback minimal)
- Tout ce qui a prouvé son utilité en V5 (RPE, sustained, adaptive decay, reverberation) est **toujours activé**
- Le code de calibration non testé est supprimé jusqu'à preuve de besoin
- Les baselines TopologyOnly/RandomBaseline restent comme modes de test, mais partagent le même pipeline

---

## 5. Plasticité destructrice sur CPU

### 5.1 Symptôme

| Backend | TopologyOnly | FullLearning | Δ |
|---------|-------------|-------------|---|
| CPU vanilla | 84.4% | 64.1% | **−20.3 pp** ❌ |
| CPU sustained | 97.3% | 87.9% | **−9.4 pp** ❌ |
| GPU sustained | 81.6% | 86.7% | **+5.1 pp** ✅ |

Sur CPU, activer la plasticité **dégrade** l'accuracy au lieu de l'améliorer. Le réseau « désapprend » par rapport à la configuration topologique.

### 5.2 Causes identifiées

- **Champ dopaminergique trop diffus** : $\lambda = 0.15$ sur $[0, 100]^3$ → rayon effectif ~6.7 unités. Toutes les arêtes reçoivent de la dopamine, pas seulement celles proches du reward center.
- **Gain plastique trop élevé** : $\eta = 3.0$ — les modifications de conductance sont massives à chaque trial.
- **Homéostatique non sélective** : $\rho \cdot (C_0 - C)$ ramène toutes les conductances vers $C_0$, effaçant les patterns utilement différenciés.
- **Timing divergent** (shader fusionné) : la fatigue/inhibition/trace sont calculées AVANT la réverbération sur GPU, APRÈS sur CPU → écart de ~5pp.

### 5.3 V6

Ce problème sera atténué par la suppression des forks (un seul pipeline) et l'amélioration de la dynamique énergétique.

---

## 6. Re-apprentissage (Remap) bloqué

### 6.1 Symptôme

Remap : 25% d'accuracy (chance aléatoire en binaire = 50%). Le réseau ne peut pas se réadapter après inversion du mapping à mi-entraînement.

### 6.2 Cause

La **consolidation irréversible** : les edges renforcés pendant la phase initiale reçoivent un flag `consolidated = true` qui les immunise contre toute modification future. Même l'oubli accéléré ($\rho_{\text{boost}} = 0.02$) ne peut pas affecter les edges consolidés.

### 6.3 V6

Mécanisme de dé-consolidation conditionnelle : un RPE fortement négatif persistant sur N trials devrait pouvoir lever le flag de consolidation.

---

## 7. Gap GPU ↔ CPU (~10pp)

### 7.1 Symptôme

TopologyOnly GPU = 81.6% vs TopologyOnly CPU = 97.3%. Écart de **15.7pp** sans aucune plasticité — la physique du réseau diffère entre les deux backends.

### 7.2 Cause

Le shader fusionné `apply_and_dissipate.wgsl` combine en un seul pass :
1. Application des influences
2. Vérification de seuil
3. Mise à jour fatigue / inhibition / trace mémoire
4. Dissipation

Sur CPU, ces opérations sont séparées et ordonnées différemment. Le timing de la fatigue par rapport à la réverbération crée un biais systématique.

### 7.3 V6

Séparation du shader fusionné en passes distinctes, alignées avec l'ordre CPU.

---

## 8. Scaling non validé

### 8.1 État

Aucun test au-delà de 50 000 nœuds. Le module `calibration.rs` implémente des lois d'adaptation multi-échelle ($G(n)$, $\gamma(n)$, etc.) mais elles n'ont jamais été vérifiées empiriquement.

### 8.2 Risque

Les paramètres optimaux pour 50k nœuds peuvent être divergents pour 500k ou 5M. Le gain effectif $G_{\text{eff}} = G \cdot \langle k \rangle \cdot \langle C \rangle$ change avec la topologie.

### 8.3 V6

Avant tout scaling, stabiliser l'énergie (§1) et supprimer les forks (§4). Le scaling est un objectif post-V6.

---

## 9. Synthèse des priorités V6

| # | Instabilité | Sévérité | Action V6 |
|---|-------------|----------|-----------|
| 1 | Ondes d'énergie tsunami | **Critique** | Inhibition latérale, activation sparse, $G_{\text{eff}} < 1$ |
| 2 | Accuracy non probante seule | **Haute** | Marge, courbes, séparabilité, tests statistiques |
| 3 | Dijkstra ≠ chemins réels | **Haute** | Traceur de signal réel |
| 4 | Forks de code proliférants | **Haute** | Suppression totale, pipeline unique |
| 5 | Plasticité CPU destructrice | Moyenne | Résolu par pipeline unique (§4) |
| 6 | Remap bloqué | Moyenne | Dé-consolidation conditionnelle |
| 7 | Gap GPU ↔ CPU | Moyenne | Shader séparé (résolu par §4) |
| 8 | Scaling non validé | Basse | Reporté post-V6 |
