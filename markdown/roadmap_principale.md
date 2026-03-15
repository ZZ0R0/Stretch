# Roadmap principale — révision post-V5
## Stretch : trajectoire corrigée à partir du système réellement obtenu

## 1. Objet

Ce document remplace la roadmap précédente à partir de la sortie réelle de la V5.

Il prend en compte :

- les aboutissements techniques effectifs de la V4 et de la V5 ;
- les instabilités découvertes pendant la V5 (dette technique, biais d'évaluation, dynamique énergétique) ;
- la vision post-V5 orientée vers un nettoyage profond et un rapprochement biologique ;
- la contrainte de versioning sans versions intermédiaires ;
- les **leçons de faisabilité** : chaque version future doit être réaliste vis-à-vis de la complexité technique réelle du projet.

Le but n'est pas de prolonger mécaniquement la roadmap antérieure, mais de la recaler sur la réalité du système. Les suppositions faites sans connaissance de cause dans les versions précédentes sont corrigées ici.

---

## 2. Ce qui est désormais acquis

### V0 — substrat dynamique minimal
- propagation locale ;
- dissipation ;
- traces ;
- plasticité locale ;
- corridors préférentiels.

### V1 — substrat spatial 3D non-grid
- graphe spatial 3D ;
- topologies KNN / radius ;
- KD-tree ;
- calibration 3D ;
- visualisation 3D ;
- formalisation mathématique.

### V2 — activité endogène régulée
- partitionnement spatial ;
- PID ;
- pacemakers ;
- consolidation structurelle ;
- activité auto-entretenue.

### V3 — dynamiques internes crédibles
- neurones E/I ;
- propagation signée ;
- PID indirect ;
- STDP ;
- budget synaptique compétitif ;
- scale 500k nœuds CPU ;
- substrat moins homogène.

### V4 — infrastructure d’apprentissage scalable
- architecture GPU-first ;
- pipeline de plasticité complet :
  STDP → éligibilité → 3 facteurs → homéostasie → consolidation → budget synaptique ;
- performance très supérieure à V3 ;
- compression mémoire forte ;
- exécution jusqu’à 5M nœuds et 57M arêtes ;
- portabilité GPU via wgpu.
### V5 — preuve d'apprentissage réelle et dynamique soutenue
- framework anti-biais topologique (Symmetric, Inversé, Remap) ;
- 3 baselines : RandomBaseline (68.8%), TopologyOnly, FullLearning ;
- preuve d'apprentissage : Δ = +84 pp en inversé CPU, Δ = +26.5 pp en inversé GPU ;
- première plasticité constructive en mode normal : Δ = +5.1 pp (GPU V5.2.1) ;
- dynamique soutenue : decay adaptatif, réverbération locale, reset partiel ;
- RPE (Reward Prediction Error) + modulation marge + oubli accéléré ;
- pipeline GPU complet : 15 phases, 14 shaders, single submit, ~9ms/tick (50k) ;
- orchestrateur unifié (process_readout) éliminant la duplication CPU/GPU sur la logique métier ;
- visualisation 3D complète (stretch-viz V5) : carte conductance, chemins Dijkstra, sparklines ;
- 13 expériences d'isolation documentées ;
- 12 composants mathématiques audités (cohérence équations ↔ CPU ↔ GPU).
---

## 3. Diagnostic post-V4

La V4 a résolu un bloc énorme d’infrastructure.
Stretch possède maintenant ses jambes : performance, échelle, pipeline, GPU compute.

Mais la V4 n’a pas encore prouvé que le système possède son cerveau.

### 3.1 Limites critiques observées

- le réseau fonctionne en régime flash & die ;
- l’activité n’est pas soutenue, mais pulsée ;
- l’accuracy 100% observée peut être expliquée par un biais topologique ;
- aucune preuve propre de chemins dopaminergiques n’a encore été démontrée ;
- les modifications de conductance restent faibles et symétriques ;
- l’éligibilité décroît trop vite par rapport au délai de reward ;
- le PID travaille sur un régime non stationnaire ;
- les paramètres ne scalent pas proprement de 50k à 5M ;
- l’infrastructure est validée, mais l’apprentissage véritable ne l’est pas encore.

---
## 3bis. Diagnostic post-V5

La V5 a franchi la Gate A : l'apprentissage structurel sur GPU est prouvé au-delà du biais topologique. Mais la construction rapide a révélé des fragilités structurelles qui doivent être traitées avant de poursuivre.

### 3bis.1 Dette technique : forks de code
Le codebase contient 13+ chemins conditionnels (CPU vs GPU, feature toggles, modes PID, calibration non testée). Ces forks produisent une divergence silencieuse d'environ 10pp entre backends et rendent impossible la validation rigoureuse.

### 3bis.2 Biais d'évaluation
L'accuracy binaire (argmax sur fenêtre 256 ticks) ne suffit pas à prouver la qualité de l'apprentissage. Il manque : marge moyenne, courbes d'apprentissage, multi-seed avec écart-type, tests de significativité statistique.

### 3bis.3 Énergie irréaliste
Le gain effectif $G_{\text{eff}} \approx 1.44 > 1$ fait que chaque tick amplifie l'énergie au lieu de l'atténuer, produisant des ondes tsunami. Un réseau biologique a typiquement $\leq 5\%$ de neurones actifs simultanément ; Stretch n'a pas de contrainte de sparsité.

### 3bis.4 Observation vs hypothèse
Dijkstra montre le chemin de coût minimal sur les conductances statiques, pas le routage réel du signal. On suppose que les chemins se forment, mais on ne les observe pas directement.

**Leçon centrale** : *observer d'abord, construire ensuite*. La V6 doit consolider avant que la V7 ne construise de nouveaux mécanismes.

---
## 4. Conséquence sur la roadmap

Le problème prioritaire n'est plus :
- la hiérarchie ;
- la spécialisation riche ;
- les assemblées ;
- la symbolisation.

Le problème prioritaire post-V5 est désormais :

> **Nettoyer, stabiliser, et vérifier** avant de construire de nouveaux mécanismes.

La V5 a prouvé l'apprentissage mais a aussi révélé que la rapidité de construction produit une dette technique invisible. La V6 doit être une version de consolidation, pas de construction.

---

## 5. Nouvelle logique de progression

```text
V4  acquis
→ V5  preuve d'apprentissage réelle + dynamique soutenue (ACQUIS)
→ V6  nettoyage technique + stabilité énergétique + rigueur de mesure + chemins réels
→ V7  chemins dopaminergiques robustes, reward prediction minimale
→ V8  hiérarchie de zones et routage multi-échelle
→ V9  spécialisation neuronale riche et canaux multiples
→ V10 assemblées dynamiques et mémoire de travail
→ V11 chaînage, prédiction temporelle et proto-raisonnement
→ V12 symbolisation et interface discrète
→ V13 boucle linguistique externe minimale
→ V14 agent textuel émergent minimal
```

La V6 est **insérée** entre la V5 et l'ancienne V6 (devenue V7). Chaque version suivante est décalée de +1.

---

## 6. Roadmap consolidée

```text
V0   substrat dynamique minimal                                acquis
V1   espace 3D non-grid                                        acquis
V2   régulation locale et activité endogène                    acquis
V3   inhibition, STDP, PID indirect                            acquis
V4   dopamine, reward, éligibilité, I/O minimales + GPU-first  acquis
V5   preuve d'apprentissage, dynamique soutenue                acquis

V6   nettoyage technique, stabilité, rigueur, chemins réels
V7   chemins dopaminergiques, mémoire guidée, reward prediction
V8   hiérarchie de zones et routage multi-échelle
V9   spécialisation neuronale riche et canaux multiples
V10  assemblées dynamiques et mémoire de travail
V11  chaînage, prédiction temporelle et proto-raisonnement
V12  symbolisation et interface discrète
V13  boucle linguistique externe minimale
V14  agent textuel émergent minimal
```

---
# 7. V5 — Preuve d'apprentissage réelle, calibration multi-échelle, dynamique soutenue — **ACQUIS**

## 7.1 Intention

La V5 devait répondre à une question unique et non négociable :

> le réseau apprend-il réellement, ou la topologie fait-elle encore le travail à sa place ?

La V5 était la version de :
- preuve ;
- calibration ;
- stabilisation dynamique ;
- diagnostic scientifique.

## 7.2 Objectifs

- éliminer les biais topologiques des tâches ;
- prouver l’existence d’un apprentissage au-delà de la géométrie ;
- stabiliser une activité plus soutenue ;
- adapter les hyperparamètres à l’échelle ;
- fournir des outils diagnostics permettant de lire ce qui est appris.

## 7.3 Transformations majeures

### A. Tâches anti-biais topologique
- I/O symétriques ;
- associations inversées ;
- re-routing ;
- inversion de mapping après pré-entraînement.

### B. Baselines strictes
- random baseline ;
- topologie-only baseline ;
- full learning baseline.

### C. Calibration multi-échelle
Hyperparamètres adaptatifs selon `n`, `extent`, `group_size`, `read_delay`, etc.

### D. Dynamique soutenue
- decay adaptatif ;
- activité réverbérante locale ;
- connexions récurrentes ;
- présentations plus longues ;
- suppression ou réduction des resets destructeurs.

### E. Outils de compréhension
- heatmaps de conductance 3D ;
- path tracer ;
- timeline eligibility / conductance ;
- analyse de clusters de co-renforcement.

## 7.4 Ce que la V5 devait prouver

- l'accuracy reste > hasard sur des tâches sans biais topologique ;
- des routes se forment effectivement entre entrée et sortie ;
- la dynamique devient moins flash & die ;
- les paramètres scalent mieux au-delà de 50k.

## 7.5 Critères d'acceptation — Bilan

| Critère | Verdict | Résultat |
|---------|---------|----------|
| Tâche anti-biais apprise | ✅ | Δ = +84 pp inversé CPU, Δ = +26.5 pp inversé GPU |
| Plasticité > baseline topology-only | ✅ | Δ = +5.1 pp en normal GPU (V5.2.1) |
| Chemins renforcés cohérents | ⚠️ | D_k positifs en inversé ; partiellement en normal |
| Dynamique soutenue stable | ✅ | Decay adaptatif + réverbération + reset partiel |
| Protocole archivé et reproductible | ✅ | 13 expériences, configs seed=42, résultats documentés |
| Calibration multi-échelle | ❌ | Implémentée mais non validée > 50k nœuds |

**Verdict : V5 validée.** Gate A franchi. Voir `V5/V5_aboutissement.md` pour le bilan détaillé.

## 7.6 Problèmes révélés par la V5

**Ces problèmes n'étaient pas anticipés dans la roadmap originale.** Ils imposent une version intermédiaire de consolidation (V6) avant de poursuivre :

1. **Accumulation de forks de code** : 13+ branches conditionnelles → divergence GPU/CPU de ~10pp, bugs silencieux, maintenance quadratique.
2. **Évaluation insuffisante** : l'accuracy sans marge, sans courbe de progression, sans test multi-seed ne prouve pas rigoureusement l'apprentissage.
3. **Ondes d'énergie tsunami** : $G_{\text{eff}} \approx 1.44 > 1$ → activation massive → éligibilité non-sélective → plasticité aveugle.
4. **Chemins non observés** : Dijkstra montre des hypothèses géométriques, pas le routing réel du réseau.

Voir `V5/V5_instabilités.md` pour l'analyse complète.

---

# 8. V6 — Nettoyage technique, stabilité énergétique, rigueur de mesure, chemins réels

## 8.1 Intention

La V6 ne construit rien de nouveau. Elle **nettoie**, **stabilise**, et **vérifie** ce que la V5 a construit.

Le constat post-V5 est clair : les fondations fonctionnent mais sont fragiles. Avant d'empiler de nouvelles capacités (chemins robustes, reward prediction, hiérarchie), il faut :
- éliminer le code mort et les forks qui génèrent des divergences silencieuses ;
- rendre le comportement énergétique biologiquement réaliste ;
- prouver rigoureusement que les métriques mesurent bien ce qu'elles prétendent ;
- observer ce que le réseau fait réellement, pas ce qu'on suppose qu'il fait.

## 8.2 Axe 1 — Suppression totale du code mort et des forks

**Principe** : pas de feature conditionnelle, tout doit être unique.

| Action | Justification |
|--------|---------------|
| Éliminer le chemin CPU dupliqué (`step_cpu`) | Un seul pipeline GPU ; CPU = fallback minimal sans parité |
| Activer RPE, sustained, adaptive decay, reverb en permanence | Tout ce qui a prouvé son utilité est toujours ON |
| Supprimer les feature toggles (margin_mod, spatial dopa, etc.) | Chaque toggle = un fork = un risque de divergence |
| Séparer le shader fusionné `apply_and_dissipate` | Élimine le gap GPU ↔ CPU (~10pp de divergence) |
| Supprimer la calibration non testée (`calibration.rs`) | Dead code : jamais validé > 50k nœuds |
| Supprimer le mode PID indirect inutilisé | Deux algorithmes PID = confusion ; garder le mode vérifié |

## 8.3 Axe 2 — Stabilité énergétique et réalisme biologique

**Problème** : $G_{\text{eff}} \approx 1.44 > 1$ → chaque étape amplifie au lieu d'atténuer → ondes tsunami.

| Action | Justification |
|--------|---------------|
| Inhibition latérale intra-tick | Les neurones inhibiteurs agissent dans le même pas de propagation |
| Activation sparse (top-k ou softmax par zone) | Limiter le % de neurones actifs simultanément (~1-5%) |
| Réduire $G_{\text{eff}} < 1$ | Par ajustement de $G$, du fan-out, ou de $\langle C \rangle$ |
| Métriques de sparsité | % nœuds actifs par tick comme métrique de monitoring |
| Benchmark biologique | Critère : ≤ 5% de neurones actifs simultanément |

## 8.4 Axe 3 — Évaluation rigoureuse de l'apprentissage

**Problème** : l'accuracy binaire (argmax sur fenêtre 256) ne prouve pas la qualité de l'apprentissage.

| Action | Justification |
|--------|---------------|
| Marge moyenne comme métrique primaire | Plus informative que l'accuracy binaire |
| Courbes d'apprentissage tick-by-tick exportées | Montrer la progression, pas juste le résultat final |
| Séparabilité des distributions P(S_c) | Prouver que la discrimination s'améliore |
| Multi-seed systématique (5 seeds, μ ± σ) | Prouver la reproductibilité |
| Tests statistiques (Welch's t-test sur Δ) | Prouver la significativité |

## 8.5 Axe 4 — Visualisation des chemins réels d'apprentissage

**Problème** : Dijkstra montre le chemin de coût minimal sur les conductances statiques, pas le routing réel.

| Action | Justification |
|--------|---------------|
| Traceur de signal réel (activation bitmap par tick) | Voir les routes que le signal emprunte réellement |
| Enregistrement des arêtes actives (pré ET post actifs) | Identifier les synapses effectivement utilisées |
| Visualisation du flux effectif input → output | Remplacer l'hypothèse Dijkstra par l'observation |
| Comparaison chemins réels vs Dijkstra | Mesurer si la cohérence topologique (CT) est prédictive |

## 8.6 Ce que doit prouver la V6

- le codebase ne contient plus de forks conditionnels (un seul chemin d'exécution) ;
- l'énergie n'explose plus en onde tsunami (≤ 5% de neurones actifs simultanément au pic) ;
- le Δ(FullLearning − TopologyOnly) est statistiquement significatif (p < 0.05, 5 seeds) ;
- les chemins réels du signal sont visualisables et cohérents avec les conductances apprises.

## 8.7 Critères d'acceptation V6

| # | Critère | Seuil |
|---|---------|-------|
| 1 | Zéro fork conditionnel dans le code | Aucun `if feature_enabled` |
| 2 | Sparsité d'activation | ≤ 5% de neurones actifs au pic |
| 3 | Δ significatif | p < 0.05 (Welch's t-test, 5 seeds) |
| 4 | Marge croissante | Marge moyenne augmente au cours de l'entraînement |
| 5 | Chemins réels visualisés | Traceur fonctionnel, overlay dans stretch-viz |
| 6 | Build propre | 0 erreurs, 0 warnings, tests verts |

---

# 9. V7 — Chemins dopaminergiques robustes, mémoire guidée, reward prediction minimale

> **Note** : cette section correspond à l'ancienne V6 de la roadmap. Elle est reportée en V7 car ses prérequis (fondations saines) sont traités par la nouvelle V6.

## 9.1 Intention

Une fois le code nettoyé, l'énergie stabilisée, et les métriques fiabilisées, la V7 peut construire :
- des chemins appris robustes (maintenant qu'on peut les observer) ;
- une mémoire guidée moins fragile ;
- une première forme de reward prediction.

## 9.2 Objectifs

- formation explicite de routes préférentielles ;
- consolidation plus robuste et plus localisée (dé-consolidation conditionnelle) ;
- meilleure résistance à l'homéostasie érosive ;
- première forme de reward prediction ou valeur locale ;
- remap fonctionnel (> 60% après inversion mid-training).

## 9.3 Prérequis (V6 atteints)

- code sans forks → un seul chemin à optimiser ;
- énergie sparse → éligibilité sélective → plasticité ciblée ;
- métriques fiables → on peut mesurer si les chemins se forment réellement ;
- traceur de signal → on peut vérifier visuellement que les routes persistent.

## 9.4 Ce que doit prouver la V7

- les chemins formés persistent assez pour être réutilisés ;
- la récompense n'est plus seulement consommée, elle commence à être anticipée ;
- la mémoire utile résiste mieux au bruit et au temps ;
- le remap fonctionne (consolidation réversible).

---

# 10. V8 — Hiérarchie de zones et routage multi-échelle

## 10.1 Intention

Une fois l'apprentissage réel, les chemins robustes, et les fondations saines établis, il devient pertinent d'introduire une architecture régionale hiérarchique.

## 10.2 Prérequis de faisabilité

- V7 atteinte : chemins persistants, remap fonctionnel, mémoire résistante ;
- énergie sparse confirmée sur >10k ticks ;
- multi-tâches (>2 classes) fonctionnel (readout N-classes déjà paramétrique) ;
- **seulement si les chemins réels observés montrent une structure spatiale exploitable**.

## 10.3 Objectifs

- micro / méso / macro-zones ;
- routage multi-échelle ;
- budgets régionaux ;
- zones spécialisées par rôle ;
- modulation descendante simple.

---

# 11. V9 — Spécialisation neuronale riche et canaux multiples

## 11.1 Intention

Après hiérarchie, on peut diversifier réellement les neurones.

## 11.2 Prérequis de faisabilité

- hiérarchie de zones stable (V8) ;
- les zones montrent une spécialisation émergente mesurable ;
- **seulement si la complexité du codebase reste gérable** (leçon V5 : chaque ajout crée de la dette).

## 11.3 Objectifs

- dépasser E/I ;
- introduire sous-types fonctionnels ;
- ajouter au moins un canal modulateur supplémentaire si utile ;
- préparer la mémoire active.

---

# 12. V10 — Assemblées dynamiques et mémoire de travail

## 12.1 Intention

Faire émerger des groupes réactivables qui peuvent porter une information au-delà du stimulus immédiat.

## 12.2 Prérequis de faisabilité

- spécialisation neuronale (V9) ;
- activation sparse maintenue ;
- **seulement si le traceur de signal (V6) montre des clusters d'activation cohérents et réactivables**.

## 12.3 Objectifs

- formation d'assemblées ;
- maintien temporaire ;
- réactivation partielle ;
- compétition entre assemblées ;
- mémoire de travail active.

---

# 13. V11 — Chaînage, prédiction temporelle et proto-raisonnement

## 13.1 Intention

Passer des assemblées isolées à leurs transitions.

## 13.2 Prérequis de faisabilité

- assemblées stables et mémoire de travail (V10) ;
- **seulement si les assemblées montrent une dynamique temporelle (activation séquentielle, pas simultanée)**.

## 13.3 Objectifs

- apprendre A→B→C ;
- produire des prédictions temporelles ;
- maintenir un contexte interne court ;
- initier des boucles de comparaison attente / résultat.

---

# 14. V12 — Symbolisation et interface discrète

## 14.1 Intention

Avant le texte, il faut une couche discrète stable.

## 14.2 Prérequis de faisabilité

- chaînage temporel (V11) ;
- **seulement si les patterns internes sont suffisamment stables et distinctifs pour être discrétisés** ;
- attention : la symbolisation est un saut conceptuel majeur. Si les assemblées sont trop bruitées, cette étape n'est pas faisable.

## 14.3 Objectifs

- vocabulaire interne minimal ;
- association motifs ↔ symboles ;
- encodeur/décodeur discret ;
- readout symbolique.

---

# 15. V13 — Boucle linguistique externe minimale

## 15.1 Intention

Relier la couche symbolique au texte externe.

## 15.2 Prérequis de faisabilité

- symbolisation stable (V12) ;
- **seulement si le readout symbolique produit des séquences non-triviales**.

## 15.3 Objectifs

- entrée textuelle minimale ;
- sortie textuelle rudimentaire ;
- micro-contexte ;
- mapping texte ↔ symboles internes.

---

# 16. V14 — Agent textuel émergent minimal

## 16.1 Intention

Construire un premier agent textuel de très bas niveau.

## 16.2 Prérequis de faisabilité

- boucle linguistique (V13) ;
- **seulement si les interactions textuelles montrent un comportement adaptatif, pas seulement des patterns mémorisés**.

## 16.3 Objectifs

- mini-conversation ;
- petite mémoire récente ;
- patrons textuels appris ;
- comportement local orienté but.

---
- mini-conversation ;
- petite mémoire récente ;
- patrons textuels appris ;
- comportement local orienté but.

---

## 17. Sprint transversal — Performance et outils

À partir de maintenant, l’axe transversal ne doit plus prioritairement être faire tourner plus vite, mais :

- maintenir la performance ;
- améliorer la visualisation ;
- renforcer les outils diagnostics ;
- automatiser les grid-search et benchmarks.

Axes transversaux recommandés :
- performance GPU continue ;
- visualisation 3D des conductances ;
- traçage des chemins **réels** (pas seulement Dijkstra) ;
- check-pointing ;
- dashboards de métriques (marge, sparsité, courbes d'apprentissage) ;
- **dette technique zéro** : chaque version doit laisser le code plus propre qu'elle ne l'a trouvé.

---

## 17bis. Invariants techniques — Sécurité de réalisation

Ces règles s'appliquent à **toutes les versions futures**. Elles agissent comme des rails de sécurité pour empêcher la dérive technique observée en V5.

### I1. Un seul chemin d'exécution

Chaque feature doit avoir **un seul chemin de code**. Pas de `if backend == "cpu"` vs `"gpu"` pour la même logique. Si un backend n'est pas prioritaire, il n'a pas de parité feature-à-feature : il fonctionne en mode dégradé explicite.

### I2. Pas de feature toggle

Si une feature est prouvée utile, elle est toujours activée. Si elle n'est pas prouvée utile, elle est supprimée. Le code ne contient pas de `if config.feature_enabled`. Exception : les modes baseline (TopologyOnly, RandomBaseline) qui désactivent la plasticité pour les protocoles expérimentaux.

### I3. Sparsité d'activation vérifiée

À partir de la V6, le % de neurones actifs simultanément est une métrique de monitoring obligatoire. Si ce % dépasse 10% en régime normal, c'est un bug, pas un paramètre à ajuster.

### I4. Métriques de confiance, pas seulement d'accuracy

Chaque version qui prétend améliorer l'apprentissage doit fournir :
- le Δ (vs TopologyOnly) ;
- la marge moyenne ;
- la reproductibilité (σ sur 5 seeds) ;
- la significativité statistique (p-value).

Une accuracy sans ces métriques ne constitue pas une preuve.

### I5. Observation avant construction

Avant d'ajouter un mécanisme (hiérarchie, assemblées, symbolisation...), il faut **observer** que le phénomène visé émerge naturellement ou qu'il manque. Le traceur de signal réel (V6) est la base de cette observation. On ne construit pas un mécanisme pour un problème qu'on n'a pas observé.

### I6. Complexité budgétée

Chaque version a un budget de complexité implicite. Si une version ajoute un mécanisme, elle doit aussi simplifier ou supprimer du code existant. La complexité nette doit rester stable ou diminuer. La V5 a montré que la complexité croissante produit de la dette exponentielle.

### I7. Gate de faisabilité avant chaque version

Avant de commencer une version V(n+1), vérifier que les prérequis techniques de la version sont réellement atteints — pas supposés atteints. Si un prérequis n'est pas rempli, la version est **reportée**, pas commencée avec un prérequis absent.

---

## 18. Gates expérimentaux corrigés

### Gate A — fin V5 — **FRANCHI** ✅

Conditions remplies :
- ✅ apprentissage au-delà du biais topologique démontré (Δ +84 pp inversé, Δ +5.1 pp normal GPU) ;
- ✅ baselines battues (FullLearning > TopologyOnly en mode inversé et normal GPU) ;
- ⚠️ chemins appris partiellement cohérents (D_k positifs en inversé, partiels en normal).

Verdicts : Gate A franchi avec réserves. Les problèmes résiduels (forks, accuracy, énergie, observation) sont traités en V6.

### Gate A' — fin V6 (NOUVEAU)

Pas de poursuite vers la V7 (chemins robustes, reward prediction) si :
- le code contient encore des forks conditionnels ;
- l'énergie n'est pas sparse (> 5% de neurones actifs au pic) ;
- le Δ n'est pas statistiquement significatif (p < 0.05, 5 seeds) ;
- les chemins réels ne sont pas visualisables.

Ce gate est **bloquant**. La V7 ne peut pas construire des chemins robustes si on ne sait pas les observer.

### Gate B — fin V10 (anciennement V9)
Pas de poursuite symbolique si :
- pas d'assemblées stables ;
- pas de mémoire de travail exploitable ;
- pas de réactivation partielle crédible ;
- **pas de structure temporelle dans les activations** (ajout post-V5).

### Gate C — fin V12 (anciennement V11)
Pas de boucle linguistique si :
- les symboles internes sont trop instables ;
- le readout discret est artificiel ;
- **la complexité du codebase a dérivé** (invariant I6 violé).

---


---

# 19. Priorité de réalisation

1. ~~**V5** — apprentissage structurel sur GPU~~ → **ACQUIS** (V5.2.1)
2. **V6** — nettoyage, stabilité, rigueur, observation → **PRIORITÉ IMMÉDIATE**
3. V7 — chemins dopaminergiques robustes, reward prediction
4. V8 — hiérarchie de zones
5. V9 — spécialisation neuronale
6. V10 — assemblées et mémoire de travail
7. V11 — chaînage et prédiction temporelle
8. V12 — symbolisation
9. V13 — boucle linguistique
10. V14 — agent textuel émergent

**Règle fondamentale** : aucune version V(n+1) ne commence tant que les critères d'acceptation de V(n) ne sont pas remplis et vérifiés.

---

# 20. Conclusion

Le projet Stretch a démontré avec la V5 qu'un réseau continu, non-spiking, peut apprendre structurellement sur GPU avec un gain mesurable. Les fondations sont posées.

Mais la V5 a aussi révélé que la rapidité de construction crée de la dette technique invisible : forks de code, métriques insuffisantes, dynamique énergétique irréaliste, hypothèses non vérifiées. Cette dette, si elle n'est pas traitée, contaminera toutes les versions suivantes.

La V6 est donc une version de **consolidation**, pas de construction. Elle applique la leçon centrale de la V5 : *observer d'abord, construire ensuite*. Les 7 invariants techniques (§17bis) et les gates progressives (§18) garantissent que chaque avancée sera bâtie sur des fondations vérifiées.

Le chemin vers l'agent textuel (V14) est long mais chaque étape inclut désormais ses prérequis de faisabilité et ses critères d'acceptation mesurables. La roadmap ne promet plus : elle conditionne.
