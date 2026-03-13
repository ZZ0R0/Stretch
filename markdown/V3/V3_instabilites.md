# Instabilités et limites V3 — Pistes pour V4+

## Résumé

La V3 résout les instabilités fondamentales de V2 (pas d'inhibition, PID omnipotent, plasticité aveugle, consolidation de masse). Cependant, elle révèle de **nouvelles limites** et laisse des **problèmes ouverts** qui conditionnent la suite de la roadmap. Ce document inventorie les instabilités résiduelles, les limites structurelles, et les mécanismes manquants.

---

## 1. Instabilités identifiées

### 1.1 Oscillations émergentes non validées

**Problème** : Le cahier des charges V3 exigeait au moins un régime oscillatoire émergent (issu de boucles E→I→E, sans pacemaker). Ce critère n'a pas été formellement validé.

**Mécanisme attendu** : Dans un circuit E→I→E récurrent local, l'excitation provoque l'inhibition qui supprime temporairement l'excitation, puis l'inhibition décroît et le cycle recommence. La fréquence dépend des constantes de temps ($\tau_{\text{fatigue}}$, $\tau_{\text{inhibition}}$, délai de propagation).

**Causes possibles de l'absence** :
- Les constantes de temps inhibitrices (inhibition_decay = 0.03) sont peut-être trop lentes ou trop rapides pour le régime oscillatoire.
- Le ratio E/I (80/20) ou le gain inhibiteur ($g_I = 0{,}8$) nécessitent un calibrage fin.
- Le PID indirect masque potentiellement les oscillations en compensant les fluctuations.
- L'absence de connexions récurrentes locales explicites (les KNN ne garantissent pas de boucles courtes).

**Pistes V3.x** :
1. Protocole d'ablation : désactiver les pacemakers et observer si des oscillations subsistent.
2. Sweep paramétrique : varier $g_I \in [0{,}5, 1{,}5]$ et inhibition_decay $\in [0{,}02, 0{,}10]$.
3. Analyse spectrale FFT sur l'énergie par zone.
4. Forcer des topologies avec boucles courtes (ex : radius_3d avec rayon court) et observer.

---

### 1.2 Consolidation encore trop rapide

**Problème** : Le budget synaptique empêche la consolidation **de masse**, mais sous forte activité PID, les arêtes les plus sollicitées atteignent encore le seuil de consolidation ($w_{\text{consol}} = 2{,}5$) relativement vite. Le problème est atténué mais pas éliminé.

**Mécanisme** : Le PID indirect maintient une activité moyenne $\bar{a} \approx 0{,}3$. Même si le budget redistribue les conductances, les arêtes sur les chemins les plus courts (plus fréquemment traversés) accumulent du renforcement Hebbien continu :

$$w_{ij}(t+1) = w_{ij}(t) + \eta_+ \times p \times (\text{coact} - c_\theta)$$

Avec $\bar{a} = 0{,}3$ et $c_\theta = 0{,}2$, le renforcement est permanent sur tout nœud actif.

**Impact** : La consolidation reste partiellement corrélée à l'activité de fond PID, pas uniquement aux événements significatifs.

**Solutions V4** :
1. **Gating événementiel** : ne compter la co-activation que lorsque $a > 2 \times a_{\text{target}}$ (événements nettement au-dessus du fond).
2. **Seuil de consolidation adaptatif** : $w_{\text{consol}} = \bar{w} + k \cdot \sigma_w$ (seuls les chemins statistiquement exceptionnels se consolident).
3. **Modulation par récompense** : la consolidation ne se produit qu'en présence d'un signal de récompense (voir vision post-V3, voies dopaminergiques).

---

### 1.3 Profil STDP symétrique

**Problème** : La V3 utilise un profil STDP symétrique ($A_+ = A_- = 0{,}005$, $\tau_+ = \tau_- = 20$). Dans le cerveau, le profil est asymétrique : le LTP est plus fort que le LTD ($A_+ > A_-$), et les constantes de temps diffèrent.

**Impact** :
- La balance LTP/LTD est neutre : autant de renforcement que d'affaiblissement sur un réseau actif aléatoirement.
- Un profil asymétrique ($A_+ > A_-$) favoriserait l'apprentissage causal en renforçant davantage les séquences causales que les anti-causales.
- Les assemblées stables nécessitent un biais LTP > LTD pour se former et se maintenir.

**Piste V4** :
- Introduire $A_+ = 0{,}008$, $A_- = 0{,}004$ (ratio 2:1 LTP:LTD).
- Tester l'impact sur la formation d'assemblées proto-stables.

---

### 1.4 Scalabilité temps réel à 500k nœuds

**Problème** : La V3 fonctionne à 500k nœuds / ~5M arêtes sur 16 threads CPU, mais le ms/tick est trop élevé pour du temps réel viz fluide.

**Causes** :
- L'itération sur ~5M arêtes (PLAST+STDP+BUD) est memory-bandwidth-bound : ~320 MB de données d'arêtes.
- La propagation target-centric (incoming_adjacency) a des patterns d'accès mémoire irréguliers pour les contributions source.
- Le rendu viz à 500k points sature le GPU intégré AMD (500k rectangles = ~1M triangles/frame).

**Goulot d'étranglement identifié** : Le diagnostic CPU vs GPU montre que sur GPU intégré AMD, le bottleneck est soit le GPU (gpu_busy_percent > 80%), soit le draw call overhead CPU (500k appels draw individuels).

**Solutions** :
1. **GPU compute (wgpu)** : porter la propagation et la plasticité sur GPU compute shaders.
2. **Instanced rendering** : remplacer 500k draw_rectangle par un seul draw instancé.
3. **LOD (Level of Detail)** : ne dessiner que les nœuds actifs ou un échantillon spatial adaptatif.
4. **Batching** : accumuler les positions/couleurs dans un buffer et faire un seul draw.

---

### 1.5 Absence de différenciation neuronale au-delà de E/I

**Problème** : La V3 n'a que deux types de neurones (E et I). Dans le cerveau, il existe des dizaines de sous-types avec des propriétés distinctes (interneurones rapides, interneurones lents, cellules pyramidales, cellules stellaires, etc.).

**Impact** : Le réseau manque de diversité fonctionnelle. Tous les neurones E se comportent identiquement, tous les neurones I idem. Pas de spécialisation locale.

**Solution V4** : Étendre `NeuronType` avec des sous-types ayant des paramètres différents (seuil, fatigue, time constants).

---

## 2. Limites structurelles

### 2.1 Un seul canal de signal

Le système ne possède qu'un seul canal de communication : l'activation (scalaire positif clamped [0, 10]). Le cerveau utilise >100 neurotransmetteurs et neuromodulateurs. En particulier :

| Canal manquant | Rôle biologique | Impact sur Stretch |
|---|---|---|
| **Dopamine** | Modulation de la plasticité, signal d'erreur de prédiction de récompense | Pas d'apprentissage par renforcement |
| **Sérotonine** | Régulation de l'humeur, modulation des time constants | Pas de régulation tonique globale |
| **Noradrénaline** | Signal de saillance, modulation du gain | Pas de mécanisme d'attention |
| **Acétylcholine** | Modulation de l'apprentissage, éveil | Pas de switch apprentissage/consolidation |

L'absence de dopamine est le verrou le plus critique (voir vision post-V3).

### 2.2 Pas de hiérarchie entre zones

Les 8 zones V3 sont toutes au même niveau hiérarchique. Il n'y a pas de :
- zones micro (colonnes locales) → méso (régions) → macro (lobes) ;
- flux ascendant (bottom-up) vs descendant (top-down) ;
- communication inter-zones structurée.

### 2.3 Pas de système d'entrée/sortie

Le système reçoit des stimuli arbitraires (injection d'activation sur des nœuds spécifiques à des ticks fixés). Il n'a pas de :
- **Interface d'entrée** : encodage sensoriel, tokenisation, projection d'entrée ;
- **Interface de sortie** : décodage, lecture d'état, prise de décision ;
- **Boucle sensori-motrice** : le système ne peut pas agir sur son environnement ni recevoir de feedback.

### 2.4 Pas de système de récompense

Le réseau apprend de manière non supervisée (Hebbien + STDP). Il n'a pas de :
- signal de récompense global ou local ;
- trace d'éligibilité pour l'apprentissage par renforcement ;
- distinction entre exploration et exploitation ;
- capacité à maximiser un critère.

### 2.5 Partitionnement spatial statique

Les zones Voronoï sont fixes à l'initialisation. Pas de re-partitionnement dynamique en fonction de l'activité ou de l'apprentissage. Cela signifie que la structure régionale est arbitraire et ne reflète pas les dynamiques émergentes.

---

## 3. Dynamiques impossibles en V3

### 3.1 Apprentissage par renforcement

**Requis** : Le système modifie ses poids en fonction d'un signal de récompense retardé pour maximiser un critère.

**Pourquoi c'est impossible** : Pas de signal de récompense, pas de trace d'éligibilité, pas de voies dopaminergiques. La STDP est locale et temporelle mais non guidée par un objectif global.

### 3.2 Mémoire de travail sélective

**Requis** : Maintenir un pattern spécifique pendant plusieurs centaines de ticks, tout en supprimant les patterns concurrents.

**Pourquoi c'est partiellement impossible** : L'inhibition crée de la compétition locale, mais il n'y a pas encore de mécanisme de maintien actif (excitation réciproque soutenue au sein d'une assemblée). Le PID indirect maintient l'activité globale mais ne peut pas cibler un pattern spécifique.

### 3.3 Prédiction temporelle

**Requis** : Le réseau anticipe un stimulus futur en se basant sur les régularités apprises.

**Pourquoi c'est impossible** : La STDP apprend la causalité temporelle (A avant B renforce A→B), mais il n'y a pas de mécanisme de réactivation prédictive : la propagation suit les arêtes renforcées, mais sans signal d'amorce il n'y a pas de prédiction spontanée.

### 3.4 Oscillations multi-bandes

**Requis** : Coexistence de rythmes lents (< 8 Hz ≈ alpha) et rapides (> 30 Hz ≈ gamma), avec modulation cross-frequency.

**Pourquoi c'est improbable** : Le système n'a qu'un seul time constant set par nœud. Les oscillations émergentes (si elles existent) auraient une seule bande de fréquence déterminée par les constantes de fatigue et d'inhibition.

---

## 4. Résumé des instabilités → solutions

| # | Instabilité V3 | Impact | Solution V4+ |
|---|---|---|---|
| I1 | Oscillations non validées | Gate V3 non levée | Sweep E/I + protocole ablation |
| I2 | Consolidation rapide | Mémoire encore trop peu sélective | Gating événementiel + modulation récompense |
| I3 | STDP symétrique | Pas de biais d'apprentissage causal | Profil asymétrique LTP > LTD |
| I4 | Scalabilité 500k | Pas de temps réel | GPU compute + instanced rendering |
| I5 | Différenciation limitée E/I | Pas de spécialisation | Sous-types neuronaux |
| I6 | Signal unique | Pas de modulation globale | Voies neuromodulatrices (dopamine) |
| I7 | Pas de hiérarchie | Pas de coordination multi-échelle | Zones hiérarchiques micro/méso/macro |
| I8 | Pas d'I/O | Pas d'interaction environnement | Interfaces entrée/sortie |
| I9 | Pas de récompense | Pas d'apprentissage guidé | Architecture reward + dopamine |
| I10 | Zones statiques | Structure arbitraire | Re-partitionnement dynamique |

---

## 5. Priorités pour V4

```
Priorité 1 — Fondamentale (débloque l'apprentissage guidé) :
  ├── Voies dopaminergiques (I6, I9)
  ├── Traces d'éligibilité + gating récompense (I2, I9)
  └── Interface d'entrée minimale (I8)

Priorité 2 — Structurelle (débloque les assemblées) :
  ├── Profil STDP asymétrique (I3)
  ├── Sous-types neuronaux riches (I5)
  └── Oscillations émergentes validées (I1)

Priorité 3 — Performance (débloque l'échelle) :
  ├── GPU compute pour 500k–1M nœuds (I4)
  └── Instanced rendering (I4)

Priorité 4 — Architecture (V5+) :
  ├── Hiérarchie de zones (I7)
  ├── Interface de sortie (I8)
  └── Zones dynamiques (I10)
```
