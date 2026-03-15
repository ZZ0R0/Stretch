# V5.0 — Modélisation Mathématique Complète

> Document de référence : décrit exactement les équations implémentées dans le code,
> le raisonnement derrière chaque choix, et les grandeurs mesurées.
> Sert de base de comparaison pour toutes les versions ultérieures.

---

## 1. Architecture du Graphe

### 1.1 Espace et topologie

Le domaine est un ensemble de $N = 50\,000$ nœuds placés uniformément dans un cube $[0, L]^3$ avec $L = 100$.
Chaque nœud $i$ a une position $\mathbf{p}_i \in \mathbb{R}^3$.

La topologie est un graphe $k$-NN dirigé : chaque nœud est relié à ses $k = 10$ plus proches voisins.
Pour chaque arête $(i \to j)$, on stocke la distance euclidienne :

$$d_{ij} = \|\mathbf{p}_i - \mathbf{p}_j\|_2$$

### 1.2 État d'un nœud

Chaque nœud $i$ au tick $t$ possède :
- $a_i(t) \in [0, 10]$ : activation
- $f_i(t) \geq 0$ : fatigue (auto-inhibition)
- $h_i(t) \geq 0$ : inhibition latérale
- $m_i(t) \geq 0$ : trace mémoire (utilisation cumulée)
- $e_i(t)$ : excitabilité, dérivée de la trace : $e_i = \sigma(m_i)$

Un nœud est « actif » si $a_i(t) > \theta$ (seuil, par défaut $\theta = 0.2$).

### 1.3 État d'une arête

Chaque arête $(i \to j)$ possède :
- $C_{ij}(t) \in [C_{\min}, C_{\max}]$ : conductance (poids synaptique), $C_{\min} = 0.1$, $C_{\max} = 5.0$
- $\mathcal{E}_{ij}(t) \in [-\mathcal{E}_{\max}, +\mathcal{E}_{\max}]$ : éligibilité ($\mathcal{E}_{\max} = 5.0$)
- $\text{consolidated}_{ij} \in \{0, 1\}$ : flag de consolidation

---

## 2. Le Pipeline d'un Tick (dans l'ordre exact du code)

### Phase 0–1 : Régulation homéostatique par zones (PID)

Le réseau est partitionné en $Z = 8$ zones (Voronoi 3D). Pour chaque zone $z$ :

$$\bar{a}_z(t) = \frac{1}{|z|} \sum_{i \in z} \mathbb{1}[a_i(t) > \theta]$$

Un PID contrôle l'activité cible $a^* = 0.3$ :

$$u_z(t) = K_p \cdot e_z(t) + K_i \cdot \int e_z + K_d \cdot \dot{e}_z, \quad e_z = a^* - \bar{a}_z$$

Le PID modifie les seuils et gains locaux des nœuds de la zone.

### Phase 2–3 : Stimulation d'entrée

Pendant la fenêtre de présentation ($T_{\text{pres}} = 8$ ticks), l'entrée de la classe $c$ injecte :

$$a_i(t) \leftarrow a_i(t) + I, \quad \forall i \in \text{Input}_c$$

avec $I = 1.5$ (intensité) et $|\text{Input}_c| = 50$ nœuds.

### Phase 4 : Propagation

Le signal se propage via les arêtes avec un **kernel spatial exponentiel** :

$$w_{ij} = \exp(-\lambda \cdot d_{ij}), \quad \lambda = 0.3$$

La contribution d'un nœud actif $i$ à ses voisins :

$$s_i = a_i \cdot g \cdot g_z \cdot \text{sign}_i$$

où :
- $g = 0.8$ (gain global)
- $g_z$ : modulation PID de la zone
- $\text{sign}_i = +1$ si excitateur, $-g_{\text{inh}}$ si inhibiteur (20% des nœuds)

L'influence reçue par le nœud $j$ :

$$\Delta a_j = \sum_{i \to j} s_i \cdot C_{ij} \cdot w_{ij}$$

**C'est ici que le kernel spatial joue un rôle fondamental :**
l'influence décroît exponentiellement avec la distance.
Pour $\lambda = 0.3$ et un voisin typique à $d = 10$ :

$$w = e^{-3.0} \approx 0.05$$

Seuls les nœuds proches ont une influence significative.

### Phase 4b : Réverbération (V5)

Avant dissipation, on ajoute un écho de l'activation précédente :

$$a_i(t) \leftarrow a_i(t) + \alpha_{\text{reverb}} \cdot a_i(t-1), \quad \alpha_{\text{reverb}} = 0.12$$

Maintient l'activité pendant les gaps inter-essais.

### Phase 5 : Dissipation

**V5 (adaptive decay)** :

$$a_i(t+1) = a_i(t) \cdot (1 - d_{\text{eff},i})$$

où :

$$d_{\text{eff},i} = d_{\text{base}} + k_{\text{local}} \cdot \bar{a}_{\text{voisins}(i)}$$

$d_{\text{base}} = 0.25$, $k_{\text{local}} = 0.3$. Les régions très actives s'atténuent plus vite (auto-régulation).

La fatigue, l'inhibition et la trace mémoire suivent leurs propres dynamiques exponentielles.

### Phase 5b : Décroissance dopamine

$$D(t+1) = D_{\text{tonique}} + (D(t) - D_{\text{tonique}}) \cdot (1 - \delta_D)$$

$D_{\text{tonique}} = 0.1$, $\delta_D = 0.15$ (phasic\_decay).

### Phase 6 : Plasticité (règle des trois facteurs)

**Uniquement si la plasticité est activée** (désactivée en mode TopologyOnly).

#### Étape 1 : STDP

Pour chaque arête $(i \to j)$ dont au moins un endpoint a récemment été actif :

$$\psi_{ij} = \begin{cases} A^+ \cdot \exp\left(-\frac{\Delta t}{\tau^+}\right) & \text{si } \Delta t > 0 \text{ (post après pre)} \\[4pt] -A^- \cdot \exp\left(\frac{\Delta t}{\tau^-}\right) & \text{si } \Delta t < 0 \text{ (pre après post)} \end{cases}$$

$A^+ = A^- = 0.008$, $\tau^+ = \tau^- = 20$ ticks, $\Delta t = t_{\text{post}} - t_{\text{pre}}$.

#### Étape 2 : Éligibilité

$$\mathcal{E}_{ij}(t+1) = \gamma \cdot \mathcal{E}_{ij}(t) + \psi_{ij}$$

$\gamma = 0.95$ (decay). L'éligibilité est une mémoire glissante de la corrélation STDP.

#### Étape 3 : Règle des trois facteurs

$$\Delta C_{ij} = \eta \cdot \delta_{D,j} \cdot \mathcal{E}_{ij}$$

avec :
- $\eta = 3.0$ (plasticity\_gain)
- $\delta_{D,j}$ : signal dopaminergique au site post-synaptique $j$

**Dopamine spatialisée** (quand $\lambda_D > 0$) :

$$\delta_{D,j} = D_{\text{phasique}} \cdot \exp(-\lambda_D \cdot \|\mathbf{p}_j - \mathbf{p}_{\text{reward}}\|)$$

$\lambda_D = 0.15$, $D_{\text{phasique}} = D(t) - D_{\text{tonique}}$.

Le centre de récompense $\mathbf{p}_{\text{reward}}$ pointe vers :
- le centroïde du **groupe cible correct** si la réponse est correcte
- le centroïde du **groupe prédit (incorrect)** si erreur (pour affaiblir les mauvais chemins)

#### Étape 4 : Homéostasie synaptique

$$C_{ij} \leftarrow C_{ij} + \rho \cdot (C_0 - C_{ij})$$

$\rho = 0.0001$, $C_0 = 1.0$. Ramène lentement les poids vers la baseline.

#### Étape 5 : Budget synaptique

Pour chaque nœud source $i$, si la somme des conductances sortantes dépasse le budget $B = 30$ :

$$C_{ij} \leftarrow C_{ij} \cdot \frac{B}{\sum_{j'} C_{ij'}}$$

Normalisation compétitive : renforcer une arête en affaiblit d'autres.

### Phase 7 : Readout

Au tick $t_{\text{read}} = t_{\text{start}} + T_{\text{pres}} + T_{\text{delay}}$ ($T_{\text{delay}} = 15$ ticks) :

$$S_c = \sum_{i \in \text{Output}_c} a_i(t_{\text{read}})$$

**Décision = argmax** :

$$\hat{c} = \arg\max_c S_c$$

Correct si $\hat{c} = c_{\text{target}}$.

### Phase 7b : Reward et dopamine

$$r = \begin{cases} +1.0 & \text{si correct} \\ -0.5 & \text{si incorrect} \end{cases}$$

Le reward met à jour la dopamine :

$$D(t) \leftarrow D(t) + r \cdot g_{\text{reward}}, \quad g_{\text{reward}} = 1.0$$

---

## 3. La Preuve d'Apprentissage V5

### 3.1 Le problème du biais topologique

Dans la géométrie Legacy (V4), $d(\text{Input}_0, \text{Output}_0) = 25$ mais $d(\text{Input}_0, \text{Output}_1) = 75$.
L'avantage de propagation est :

$$\frac{w(25)}{w(75)} = \frac{e^{-7.5}}{e^{-22.5}} = e^{15} \approx 3.3 \times 10^6$$

La topologie seule résout la tâche. L'accuracy mesure le **biais géométrique**, pas l'apprentissage.

### 3.2 Solution V5 : la géométrie symétrique

Placement en croix autour du centre :
- Inputs sur l'axe Y : $(50, 30, 50)$ et $(50, 70, 50)$
- Outputs sur l'axe Z : $(50, 50, 30)$ et $(50, 50, 70)$

Pour le mapping naturel [0,1] :
$$d(\text{In}_0, \text{Out}_0) \approx 29.2, \quad d(\text{In}_0, \text{Out}_1) \approx 30.0$$

Les deux outputs sont quasi-équidistants de chaque input. Mais un biais résiduel existe :

$$\frac{w(29.2)}{w(30.0)} = e^{0.3 \times 0.8} = e^{0.24} \approx 1.27$$

L'avantage n'est plus $3.3 \times 10^6$ mais 27% — suffisant pour biaiser en l'absence d'apprentissage.

### 3.3 Solution V5 : le test anti-biais par inversion du mapping

**Principe** : garder la même géométrie mais inverser la correspondance cible.

Mapping inversé [1,0] : la classe 0 doit activer le output 1 (le plus éloigné), la classe 1 doit activer le output 0 (le plus éloigné).

**La topologie travaille contre la cible.** Si le réseau résout quand même → l'apprentissage est réel.

### 3.4 Résultats de la matrice 2×2

| Condition | TopologyOnly | FullLearning | $\Delta$ |
|---|---|---|---|
| Normal [0,1] | 97.7% | ~83% | $-14.9$ pp |
| Inversé [1,0] | **2.3%** | **86.3%** | **+84.0 pp** |

**Vérification mathématique de cohérence** :

$$A_{\text{topo, normal}} + A_{\text{topo, inversé}} = 97.7\% + 2.3\% = 100\%$$

C'est exact à l'essai près : chaque essai que la topologie « gagne » en normal, elle le « perd » en inversé. Les deux baselines sont le symétrique exact l'une de l'autre. Ceci confirme que le biais est purement géométrique et que la mesure est cohérente.

**L'apprentissage** :
- En inversé, le réseau part à 2.3% (topologie hostile) et monte à 86.3%.
- Le $\Delta = +84.0$ pp est la mesure **nette** de l'effet de la plasticité.
- Ce ne peut pas être un artefact : la topologie est fixe et contraire.

---

## 4. Métriques Diagnostiques V5

### 4.1 RouteScore (RS)

On cherche le chemin de conductance maximale entre un groupe input $I_k$ et un groupe output $O_c$ via Dijkstra sur les coûts transformés :

$$\text{cost}(i \to j) = \ln\left(\frac{C_{\max}}{C_{ij}}\right)$$

Propriétés :
- $C_{ij} = C_{\max} \Rightarrow \text{cost} = 0$ (arête parfaite)
- $C_{ij} \to 0^+ \Rightarrow \text{cost} \to +\infty$ (arête bloquée)
- Tous les coûts $\geq 0$ (Dijkstra valide)

Le RouteScore est la **somme des conductances** le long du meilleur chemin trouvé :

$$\text{RS}(I_k \to O_c) = \sum_{(i,j) \in \text{path}} C_{ij}$$

**Important** : un RS plus élevé ne signifie pas « meilleur chemin ». Le RS est juste la somme des poids sur le chemin trouvé par le critère de minimisation de $\sum \ln(C_{\max}/C_{ij})$. Un chemin avec plus de hops aura un RS plus élevé même si chaque arête est modeste.

### 4.2 Hops

Le nombre de **hops** est le nombre d'arêtes dans le chemin Dijkstra :

$$H = |\text{path}| - 1 \quad \text{(nombre de nœuds - 1)}$$

C'est la distance topologique (en nombre de sauts) entre l'input et l'output. Si hops = 7, le signal doit traverser 7 arêtes pour aller de la source à la cible.

**Biais potentiel** : les hops ne sont PAS directement liés à l'efficacité de la propagation. Le readout mesure l'activation au tick $t_{\text{read}}$ qui dépend de la propagation parallèle à travers TOUS les chemins, pas seulement le chemin Dijkstra. Le diagnostic trace UN chemin optimal, mais le signal réel emprunte des millions de chemins simultanément.

### 4.3 Bottleneck

$$B = \min_{(i,j) \in \text{path}} C_{ij}$$

Le maillon le plus faible du chemin. Un bottleneck faible signifie qu'il existe une arête étroite même sur le meilleur chemin.

### 4.4 Indice Directionnel $D_k$

$$D_k = \text{RS}(I_k \to O_{\text{target}}) - \text{RS}(I_k \to O_{\text{compétiteur}})$$

- $D_k < 0$ : l'apprentissage a créé un chemin de moindre résistance vers la cible (le RS est plus faible car le chemin est plus court/direct)
- $D_k > 0$ : le RS vers la cible est plus élevé (chemin plus long, possiblement plus de hops)

**Attention** : D n'est PAS un prédicteur direct de la performance. Voir section 5.

### 4.5 Cohérence Topologique $CT$

$$CT = \text{corr}_{\text{Pearson}}\left(\Delta C_{ij}, \; \mathbb{1}[\text{arête sur chemin utile}]\right)$$

CT > 0 signifie que les arêtes dont la conductance a augmenté sont préférentiellement celles sur les chemins de diagnostic. Mais voir les limites en section 5.

### 4.6 Sustain Ratio

$$SR = \frac{\bar{E}_{\text{inter-trial}}}{\bar{E}_{\text{pic}}}$$

Mesure la rémanence de l'activité entre les essais. Un SR élevé indique que l'énergie persiste (réverbération, traces mémoire).

---

## 5. Limites et Biais Potentiels (Analyse Critique Honnête)

### 5.1 Le diagnostic Dijkstra ≠ le mécanisme réel

Le chemin Dijkstra est un **diagnostic post-hoc** : il cherche un chemin optimal sur le graphe final. Mais pendant la simulation, le signal ne suit pas un chemin unique. Il se propage en parallèle à travers TOUS les voisins, pondéré par $w_{ij} \cdot C_{ij}$. L'activation résultante à un output est la somme de contributions d'un très grand nombre de chemins.

**Conséquence** : Le RouteScore peut être trompeur. Un RS élevé sur le chemin Dijkstra ne signifie pas que le signal passe par là. Similarly, un D positif ne signifie pas forcément que la cible est défavorisée.

### 5.2 Ce qui est fiable vs. ce qui ne l'est pas

**Fiable** (mesure directe, pas d'intermédiaire) :
- L'accuracy finale : c'est le comptage brut des décisions correctes
- Le $\Delta$ entre conditions : c'est une soustraction de mesures fiables
- La complémentarité topo (97.7% + 2.3% = 100%) : contrôle de cohérence

**Indicatif** (utile mais indirect) :
- RouteScore, hops, bottleneck : diagnostics structurels, mais ne prédisent pas directement l'accuracy
- Directional index D : corrélé à l'apprentissage mais pas causal
- CT : corrélation ≠ causalité

**Potentiellement biaisé** :
- Aucune de nos mesures actuelles n'est biaisée de manière à gonfler artificiellement l'accuracy. L'accuracy est un simple comptage `decision == target`. Le test inversé est le gant : si l'accuracy était un artefact, elle ne pourrait pas être élevée quand la topologie est hostile.

### 5.3 Biais de stochasticité (rayon)

Rayon (parallélisme) introduit un non-déterminisme dans l'ordre des réductions. Deux exécutions avec la même seed et le même config donnent ~83% et ~86% par exemple. Ce bruit est de l'ordre de ±5 pp. Il ne change pas les conclusions qualitatives (le $\Delta$ de +84 pp est robuste).

---

## 6. Résumé des Grandeurs Clés

| Symbole | Valeur | Rôle |
|---|---|---|
| $N$ | 50 000 | Nombre de nœuds |
| $k$ | 10 | Voisins par nœud |
| $\lambda$ | 0.3 | Décroissance spatiale du kernel |
| $\theta$ | 0.2 | Seuil d'activation |
| $I$ | 1.5 | Intensité d'injection |
| $T_{\text{pres}}$ | 8 ticks | Durée de présentation |
| $T_{\text{delay}}$ | 15 ticks | Délai avant lecture |
| $A^+, A^-$ | 0.008 | Amplitude STDP |
| $\tau^+, \tau^-$ | 20 ticks | Constantes STDP |
| $\gamma$ | 0.95 | Decay éligibilité |
| $\eta$ | 3.0 | Gain plasticité |
| $\lambda_D$ | 0.15 | Decay spatial dopamine |
| $D_{\text{tonique}}$ | 0.1 | Dopamine baseline |
| $r^+, r^-$ | +1.0, -0.5 | Rewards |
| $B$ | 30.0 | Budget synaptique |
| $C_0$ | 1.0 | Conductance baseline |
| $\rho$ | 0.0001 | Rate homéostasie |