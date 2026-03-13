# Instabilités et limites V2 — Pistes pour V3+

## Résumé

Le système V2 résout le problème fondamental de V1 (absence d'activité endogène) grâce au PID. Mais il introduit de **nouvelles instabilités** et révèle des **limites structurelles** qui empêchent l'émergence de comportements cognitifs plus complexes. Ce document inventorie les problèmes identifiés, les mécanismes manquants, et les pistes architecturales pour les versions futures.

---

## 1. Instabilités identifiées

### 1.1 Consolidation de masse — Absence de sélectivité

**Problème** : Le PID maintient une activité homogène $\bar{a} \approx a_{\text{target}} = 0{,}3$ dans chaque zone. Cette activité distribuée génère de la co-activation sur **toutes** les arêtes. Avec un seuil de consolidation $w_{\text{consol}} = 2{,}0$–$2{,}5$, la quasi-totalité des ~577 000 arêtes franchit le seuil en quelques centaines de ticks.

**Mécanisme** :

$$\forall (i,j) : \bar{a}_i \approx 0{,}3 > c_\theta = 0{,}2 \implies \text{renforcement continu}$$

$$w_{ij}(t) \to w_{\max} = 5{,}0 \quad \text{en } \sim \frac{w_{\max} - w_0}{\eta_+ \cdot p \cdot (\bar{a} - c_\theta)} \approx 400 \text{ ticks}$$

Une fois toutes les conductances au max et stables pendant $T_{\text{consol}} = 50$ ticks → consolidation totale.

**Impact** : La mémoire structurelle perd tout sens. Si tout est consolidé, rien n'est mémorisé sélectivement. La consolidation est censée marquer les **chemins importants**, pas tout le réseau.

**Causes profondes** :
- Le PID crée une activité **trop uniforme** spatialement.
- Le seuil de co-activation $c_\theta = 0{,}2$ est trop bas par rapport à l'activité PID ($\bar{a} = 0{,}3$).
- Il n'y a pas de mécanisme de **compétition** entre arêtes : rien n'empêche le renforcement simultané de toutes les connexions.

**Solutions possibles** :
1. **Seuil de consolidation adaptatif** : $w_{\text{consol}} = f(\bar{w})$ — le seuil monte avec la conductance moyenne du réseau.
2. **Normalisation Hebbienne** : imposer que la somme des conductances sortantes par nœud soit bornée (normalisation par ligne).
3. **Inhibition locale** : des neurones inhibiteurs qui suppriment l'activité de certains voisins → crée du contraste spatial.
4. **Consolidation compétitive** : limiter le nombre d'arêtes consolidables par nœud (quota).
5. **Décorrelation PID / plasticité** : ne compter la co-activation que pour les activations **au-dessus** d'un seuil plus élevé ($c_\theta = 0{,}5$ par exemple), de sorte que seuls les événements forts (stimulus, propagation active) comptent, pas l'activité de fond PID.

### 1.2 Homogénéité spatiale de l'activité PID

**Problème** : Le PID, par design, cherche à amener toutes les zones à la même consigne $a_{\text{target}}$. Le résultat est un réseau où l'activité est **spatialement plate** : pas de gradients, pas de régions plus actives que d'autres, pas de "topographie" fonctionnelle.

$$\bar{a}_1(t) \approx \bar{a}_2(t) \approx \ldots \approx \bar{a}_K(t) \approx a_{\text{target}}$$

**Impact** : Pas de **spécialisation régionale**. Un cerveau n'a pas la même activité dans toutes ses régions — certaines sont plus actives, d'autres au repos. L'uniformité empêche l'émergence de "zones fonctionnelles" distinctes.

**Solutions possibles** :
1. **Consignes différenciées** par zone : $a_{\text{target},k}$ différent pour chaque zone.
2. **Consignes dynamiques** : la consigne évolue en fonction de l'activité globale ou d'un signal de récompense.
3. **Budget métabolique** : chaque zone a un budget d'énergie limité, forçant une allocation compétitive.

### 1.3 PID omnipotent — Absence de dynamique de propagation autonome

**Problème** : Le PID injecte directement de l'activation dans chaque nœud. Les ondes de propagation (héritage V0/V1) sont noyées par l'injection PID. La propagation naturelle via les arêtes joue un rôle minoritaire dans la dynamique.

**Preuve** : Si on désactive le PID après stabilisation, l'activité s'éteint en ~20 ticks (retour au comportement V1). Cela montre que l'activité est **entretenue par le PID**, pas par la propagation récurrente.

**Impact** : Le réseau n'a pas de dynamique de propagation autonome. La "cognition" V2 est essentiellement le PID qui maintient un fond lumineux, pas le réseau qui s'auto-entretient.

**Solutions possibles** :
1. **PID indirect** : au lieu d'injecter de l'activation, le PID ajuste le seuil d'activation $\theta$ ou le gain de propagation $g$ localement. Le réseau doit encore propager pour être actif.
2. **PID plus doux** : réduire $u_{\max}$ et $K_p$ pour que le PID soit un "coup de pouce" plutôt qu'un driver principal.
3. **Boucles récurrentes** : introduire des connexions récurrentes locales qui permettent un maintien par la propagation elle-même.

### 1.4 Saturation résiduelle dans les zones pacemaker

**Problème** : Le pacemaker injecte jusqu'à $A + o = 0{,}3 + 0{,}5 = 0{,}8$ au pic. Le nœud pacemaker et ses voisins immédiats atteignent fréquemment $a \approx 10$ (saturation), déclenchant la fatigue maximale. Le problème de "zone morte" V1 (I1) persiste localement autour des pacemakers.

**Impact** : Les pacemakers créent des "cratères de fatigue" qui inhibent la propagation dans leur voisinage immédiat après chaque pic.

**Solutions possibles** :
1. **Pacemakers distribués** : au lieu d'injecter dans un seul nœud, distribuer l'oscillation sur un groupe de nœuds voisins (dilution).
2. **Fatigue non-linéaire (Hill)** : $f_i(t+1) = f_i + \gamma_f \frac{a_i^2}{a_i^2 + K_f^2} - \rho_f \cdot f_i$. La saturation de Hill empêche la fatigue de monter indéfiniment.

### 1.5 Absence de neurones inhibiteurs

**Problème** : Tous les nœuds sont excitateurs. L'inhibition V2 est un mécanisme interne à chaque nœud ($h_i$ qui monte avec l'activation propre), pas une interaction entre nœuds. Il n'y a pas de nœud qui, en s'activant, **supprime** l'activité de ses voisins.

**Impact fondamental** : Sans inhibition inter-neuronale :
- Pas de **winner-take-all** : plusieurs assemblées peuvent être actives simultanément sans compétition → pas de sélection.
- Pas de **contraste spatial** : l'activité PID est uniforme car rien ne crée de "trous" dans le pattern d'activation.
- Pas de **synchronisation gamma** : les oscillations rapides (>20 Hz) dans le cerveau reposent sur des circuits E→I→E (excitation → inhibition → excitation).
- Pas de **rythmes oscillatoires naturels** : les oscillations V2 sont imposées par les pacemakers, pas émergentes de la dynamique E/I.

Le cerveau utilise ~20% de neurones inhibiteurs (GABAergiques). C'est un **mécanisme fondamental**, pas un ajout optionnel.

### 1.6 Plasticité temporellement aveugle

**Problème** : La plasticité Hebbienne V2 est purement **corrélative** : si deux nœuds connectés sont actifs en même temps, la conductance monte. Mais l'**ordre temporel** n'est pas pris en compte.

Dans le cerveau, la STDP (Spike-Timing Dependent Plasticity) différencie :

$$\Delta t = t_{\text{post}} - t_{\text{pré}} > 0 \implies \text{renforcement (LTP)}$$
$$\Delta t = t_{\text{post}} - t_{\text{pré}} < 0 \implies \text{affaiblissement (LTD)}$$

Sans cela, le système ne peut pas apprendre de **séquences** : la causalité temporelle "A puis B" est indistinguable de "B puis A".

**Impact** :
- Pas d'apprentissage de séquences (V5 — mémoire procédurale sera impossible).
- Pas de prédiction (le réseau ne sait pas quel nœud a été actif en premier).
- La plasticité est symétrique : si A→B est renforcé, B→A l'est aussi.

---

## 2. Dynamiques impossibles en V2

### 2.1 Assemblées neuronales stables

**Requis** : Un groupe de nœuds co-actifs qui s'auto-entretiennent par excitation réciproque, tout en inhibant les nœuds extérieurs (compétition).

**Pourquoi c'est impossible** :
- Pas d'inhibition latérale → pas de frontière entre assemblées.
- Le PID uniformise l'activité → pas de "groupes" distincts.
- La propagation est omnidirectionnelle → pas de circuits récurrents sélectifs.

### 2.2 Compétition entre patterns

**Requis** : Que deux patterns d'activation entrent en compétition, un seul survivant (winner-take-all).

**Pourquoi c'est impossible** : Sans inhibition inter-neuronale, tous les patterns coexistent. Deux stimuli simultanés produisent une superposition linéaire, pas une compétition.

### 2.3 Séquences apprises

**Requis** : Qu'un pattern A déclenche automatiquement un pattern B après apprentissage de la séquence A→B.

**Pourquoi c'est impossible** : La plasticité corrélative renforce A↔B symétriquement. B déclencherait aussi A — pas de directionnalité causale.

### 2.4 Oscillations émergentes (non imposées)

**Requis** : Des oscillations qui naissent de la dynamique du réseau sans pacemaker externe.

**Pourquoi c'est impossible** : Démontré en V1 (I6), et toujours valable : le PID entretient l'activité mais de façon tonique, pas oscillatoire. Les oscillations V2 sont entièrement imposées par les pacemakers.

**Ce qui manque** : Des boucles E→I→E récurrentes qui oscillent naturellement à la fréquence déterminée par les constantes de temps inhibitrices.

### 2.5 Mémoire de travail

**Requis** : Un pattern d'activation maintenu temporairement (quelques centaines de ticks) puis oublié, indépendamment de la mémoire structurelle.

**Pourquoi c'est partiellement impossible** : Le PID maintient une activité de fond, mais pas un pattern spécifique. La trace mémoire ($\tau = 138$ ticks de demi-vie) donne une persistance, mais aucun mécanisme ne peut **réactiver** un pattern à partir de sa trace seule.

---

## 3. Limites architecturales

### 3.1 Un seul type de signal

Le système ne possède qu'un seul canal de communication : l'activation. Le cerveau utilise >100 neurotransmetteurs et neuromodulateurs aux effets distincts (glutamate = excitation, GABA = inhibition, dopamine = modulation de plasticité, etc.).

### 3.2 Pas de hiérarchie

Les zones V2 sont toutes au même niveau. Pas de hiérarchie micro → méso → macro. Pas de flux d'information ascendant/descendant.

### 3.3 Partitionnement statique

Les zones Voronoï sont fixes à l'initialisation. Pas de re-partitionnement dynamique, pas de zones qui fusionnent ou se scindent en fonction de l'activité.

### 3.4 Scalabilité CPU

À 50k nœuds / 577k arêtes, le système tourne à ~21 ms/tick (1 CPU). Pour atteindre des dynamiques hiérarchiques riches, il faudrait 500k–1M nœuds, ce qui nécessiterait ~200–400 ms/tick en CPU pur — trop lent pour la visualisation temps réel.

---

## 4. Résumé des instabilités → solutions

| # | Instabilité V2 | Impact | Solution V3+ proposée |
|---|---|---|---|
| I1 | Consolidation de masse | Mémoire non sélective | Normalisation Hebbienne + inhibition |
| I2 | Homogénéité PID | Pas de spécialisation régionale | Consignes différenciées + budget métabolique |
| I3 | PID omnipotent | Pas de propagation autonome | PID indirect (ajuste θ ou g, pas a) |
| I4 | Saturation pacemaker | Cratères de fatigue | Pacemakers distribués + fatigue Hill |
| I5 | Pas d'inhibition | Pas de compétition / sélection | Neurones inhibiteurs (~20%) |
| I6 | Plasticité aveugle | Pas d'apprentissage séquentiel | STDP (timing-dépendant) |
| I7 | Signal unique | Pas de modulation | Neuromodulation (≥2 canaux) |
| I8 | Pas de hiérarchie | Pas de coordination multi-échelle | Zones hiérarchiques (V3) |
| I9 | Partitionnement fixe | Pas d'adaptation structurelle | Zones dynamiques |
| I10 | Scalabilité CPU | Limite à ~50k nœuds temps réel | Parallélisation GPU |

---

## 5. Priorités pour V3

```
Priorité 1 — Fondamentale (sans quoi V4+ est impossible) :
  ├── Neurones inhibiteurs (I5) — 20% du réseau
  ├── PID indirect (I3) — ajuste le seuil, pas l'activation
  └── STDP (I6) — plasticité temporellement causale

Priorité 2 — Structurelle (nécessaire pour V4) :
  ├── Normalisation Hebbienne ou consolidation compétitive (I1)
  ├── Consignes différenciées par zone (I2)
  └── Hiérarchie de zones micro/méso (I8)

Priorité 3 — Performance (nécessaire pour V4+) :
  └── Sprint GPU pour 500k–1M nœuds (I10)
```
