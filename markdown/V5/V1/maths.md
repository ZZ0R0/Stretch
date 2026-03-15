# V5.1 — Modélisation Mathématique (Additions)

> Ce document complète `V0/maths.md` avec les équations introduites ou rendues
> explicites pendant la phase V5.1 (visualisation, diagnostics visuels, tests complémentaires).
> Le modèle de base (architecture, tick pipeline, plasticité, readout) reste inchangé.

---

## 1. Projection 3D → 2D (Visualiseur)

### 1.1 Rotation Y puis X

Le visualiseur projette les positions 3D des nœuds dans le plan écran via deux rotations successives.

Soit un nœud de position $\mathbf{p} = (x, y, z)$, et les angles de vue $\phi_Y$ (rotation autour de l'axe Y) et $\phi_X$ (rotation autour de l'axe X).

**Rotation Y** (plan XZ) :

$$x_r = x \cos\phi_Y + z \sin\phi_Y$$
$$z_r = -x \sin\phi_Y + z \cos\phi_Y$$

**Rotation X** (plan YZ') :

$$y_r = y \cos\phi_X - z_r \sin\phi_X$$

Les coordonnées écran sont $(x_r, y_r)$.

### 1.2 Mise à l'échelle et centrage

Soit $\{(x_r^{(i)}, y_r^{(i)})\}_{i=1}^N$ l'ensemble des projections. On calcule :

$$x_{\min} = \min_i x_r^{(i)}, \quad x_{\max} = \max_i x_r^{(i)}$$
$$y_{\min} = \min_i y_r^{(i)}, \quad y_{\max} = \max_i y_r^{(i)}$$

Le facteur d'échelle uniforme (pour préserver les proportions) :

$$s = \min\left(\frac{W_{\text{usable}}}{x_{\max} - x_{\min}}, \; \frac{H_{\text{usable}}}{y_{\max} - y_{\min}}\right)$$

Les coordonnées écran finales du nœud $i$ :

$$\text{screen}_x(i) = o_x + (x_r^{(i)} - x_{\min}) \cdot s$$
$$\text{screen}_y(i) = o_y + (y_r^{(i)} - y_{\min}) \cdot s$$

où $(o_x, o_y)$ est l'offset de centrage.

Ceci est encapsulé dans la structure `ProjCtx` : les coordonnées projetées et les paramètres de transformation sont calculés une seule fois par frame et réutilisés par toutes les couches de rendu (nœuds, chemins, arêtes, marqueurs I/O).

---

## 2. Conductance Moyenne par Nœud

### 2.1 Définition

Le mode de visualisation « Conductance » (touche `4`) colore chaque nœud par sa **conductance sortante moyenne** :

$$\bar{C}_i = \frac{1}{\deg_{\text{out}}(i)} \sum_{j : (i \to j) \in E} C_{ij}$$

où $\deg_{\text{out}}(i)$ est le degré sortant du nœud $i$ dans le graphe $k$-NN.

### 2.2 Interprétation

- $\bar{C}_i = C_0 = 1.0$ : nœud non modifié par la plasticité
- $\bar{C}_i > 1.0$ : nœud dont les arêtes sortantes ont été **renforcées** en moyenne
- $\bar{C}_i < 1.0$ : nœud dont les arêtes sortantes ont été **affaiblies** en moyenne

Le budget synaptique ($B = 30$, $k = 10$ voisins) impose $\sum_j C_{ij} \leq 30$, donc $\bar{C}_i \leq 3.0$ en pratique. Si une arête est fortement renforcée, les autres sont affaiblies proportionnellement (compétition synaptique).

### 2.3 Normalisation pour le rendu

La couleur est attribuée par la palette thermique `heat_color(t)` avec :

$$t = \frac{\bar{C}_i}{\max_j \bar{C}_j}$$

Ce rendu permet de visualiser les « corridors de conductance » — les régions du réseau où la plasticité a concentré les poids.

---

## 3. Sélection des Arêtes Top Conductance

### 3.1 Critère de déviation

L'overlay « top edges » (touche `C`) sélectionne les $K = 500$ arêtes les plus modifiées par rapport à la conductance baseline. Le critère est la **déviation absolue** :

$$\delta_{ij} = |C_{ij} - C_0|, \quad C_0 = 1.0$$

Les arêtes sont triées par $\delta_{ij}$ décroissant, et les $K$ premières sont affichées.

### 3.2 Coloration

Chaque arête affichée est colorée par sa conductance normalisée :

$$t = \frac{C_{ij}}{C_{\max}^{\text{top}}}, \quad C_{\max}^{\text{top}} = \max_{(i,j) \in \text{top-}K} C_{ij}$$

L'opacité est modulée :

$$\alpha = 0.15 + 0.5 \cdot t$$

Les arêtes de haute conductance sont opaques et chaudes (jaune/rouge), les arêtes de faible conductance sont translucides et froides (bleu/cyan).

### 3.3 Filtre

Les arêtes avec $\delta_{ij} \leq 0.01$ sont ignorées (pas de modification significative). Ceci élimine le bruit de l'homéostasie quand peu de plasticité a eu lieu.

---

## 4. Détection de Clusters de Co-renforcement

### 4.1 Principe

Un nœud est considéré « cluster » s'il apparaît sur **au moins 2** chemins Dijkstra tracés. Formellement :

$$\text{cluster}(i) = \mathbb{1}\left[\sum_{k=0}^{K-1} \mathbb{1}[i \in \text{path}_k] \geq 2\right]$$

où $\text{path}_k$ est le chemin Dijkstra optimal de la classe d'entrée $k$ vers sa cible.

### 4.2 Interprétation

Ces nœuds sont des **hubs de convergence** : ils se trouvent sur les routes optimales de plusieurs classes. Dans un réseau à 2 classes, un nœud cluster est un point de passage commun — ce qui peut signifier :

- Un **goulot d'étranglement** topologique (la géométrie force le passage)
- Un **carrefour appris** (la plasticité a renforcé les arêtes vers ce nœud pour les deux classes)

### 4.3 Rendu

Les nœuds cluster sont affichés en **jaune** avec un rayon 2.5× supérieur aux nœuds normaux, au-dessus de la couche des chemins.

---

## 5. Accuracy Glissante (Sparkline)

### 5.1 Suivi

L'accuracy est échantillonnée à chaque évaluation de trial. Si $E(t)$ est le nombre d'évaluations au tick $t$ :

$$A(t) = \frac{\text{correct}(t)}{E(t)}$$

La sparkline affiche les **100 dernières évaluations** dans un graphe normalisé $[0, 1]$ avec une ligne de base à 50%.

### 5.2 Rendu adaptatif

Si plus de 100 points sont disponibles, seuls les 100 derniers sont affichés (fenêtre glissante). Les points sont interpolés linéairement sur la largeur du sparkline :

$$x_k = \frac{k}{n} \cdot W_{\text{spark}}, \quad y_k = H_{\text{spark}} \cdot (1 - A_k)$$

où $k \in [0, n-1]$ est l'index dans la fenêtre et $n = \min(100, |\text{history}|)$.

---

## 6. Timeline de Conductance

### 6.1 Échantillonnage

La conductance moyenne du réseau est échantillonnée toutes les 10 évaluations :

$$\bar{C}(t) = \frac{1}{|E|} \sum_{(i,j) \in E} C_{ij}(t)$$

Cet échantillonnage est aligné sur les snapshots de métriques générés par le moteur (`metrics.snapshots.last().mean_conductance`).

### 6.2 Rendu

La timeline normalise les valeurs entre $[\bar{C}_{\min}, \bar{C}_{\max}]$ observés :

$$t_k = \frac{\bar{C}(k) - \bar{C}_{\min}}{\bar{C}_{\max} - \bar{C}_{\min}}$$

Une conductance moyenne croissante indique un renforcement net (la somme des augmentations dépasse les diminutions). Une conductance stable autour de $C_0 = 1.0$ indique que l'homéostasie domine.

---

## 7. RandomBaseline : Modèle Statistique

### 7.1 Condition

En mode `RandomBaseline`, toutes les conductances sont initialisées uniformément :

$$C_{ij} \sim \mathcal{U}[C_{\min}^{\text{rand}}, C_{\max}^{\text{rand}}], \quad C_{\min}^{\text{rand}} = 0.1, \; C_{\max}^{\text{rand}} = 5.0$$

La plasticité est **désactivée** (pas de STDP, pas de reward, pas de mise à jour des conductances).

### 7.2 Propagation sous poids aléatoires

Le signal reçu par un nœud output $j$ de classe $c$ est :

$$S_c = \sum_{i \in \text{Output}_c} a_i(t_{\text{read}})$$

où $a_i$ résulte de la propagation multi-hop à travers des arêtes de poids $w_{ij} \cdot C_{ij}$, avec $C_{ij}$ aléatoire.

### 7.3 Résultat et interprétation

**Accuracy mesurée : 68.8%** (344/500 en Symmetric).

Ce résultat est supérieur au hasard pur (50%) car :
1. Le kernel spatial $w_{ij} = e^{-\lambda d_{ij}}$ crée une structure de propagation même avec des poids aléatoires
2. Les 50 nœuds par groupe I/O ne sont pas parfaitement symétriques (variance de placement)
3. Les poids aléatoires $\mathcal{U}[0.1, 5.0]$ créent des « routes accidentelles » — certains chemins sont aléatoirement plus conducteurs que d'autres

Le 68.8% est la **borne inférieure du biais structurel** combiné (géométrie + poids aléatoires). L'apprentissage réel doit produire une accuracy significativement supérieure en mode inversé, ce qui est le cas (86.3% vs 68.8% = +17.5 pp au-dessus de la baseline aléatoire).

---

## 8. Remap : Dynamique de Re-routage

### 8.1 Protocole

Le mode Remap alterne deux phases :

- **Phase 1** (ticks $[0, T_{\text{remap}})$) : mapping normal $[0 \to 0, 1 \to 1]$
- **Phase 2** (ticks $[T_{\text{remap}}, T_{\text{total}})$) : mapping inversé $[0 \to 1, 1 \to 0]$

Avec $T_{\text{remap}} = 5000$ et $T_{\text{total}} = 10000$.

### 8.2 Dynamique attendue

En phase 1, la plasticité renforce les arêtes le long des chemins naturels (topologiquement favorisés). Les conductances $C_{ij}$ augmentent sur les routes $\text{In}_k \to \text{Out}_k$.

Au moment du remap ($t = T_{\text{remap}}$), les cibles s'inversent. Les routes renforcées deviennent **adverses** — elles propagent le signal vers le mauvais output. Le réseau doit :
1. Affaiblir les anciennes routes (via reward négatif → $\delta_D < 0$ → $\Delta C < 0$)
2. Renforcer les nouvelles routes (via les rares succès → $\delta_D > 0$)

### 8.3 Résultat et analyse

**Accuracy globale : 50.0%** (250/500). Ce résultat moyen masque une asymétrie :

- Phase 1 : accuracy élevée (topologie + apprentissage convergent)
- Phase 2 : accuracy basse (anciennes routes dominent, re-learning insuffisant)

La balance à 50% est cohérente : ce qu'on gagne en phase 1 est perdu en phase 2, presque exactement. Le réseau n'a pas eu assez de ticks pour re-router les 5000 ticks d'apprentissage accumulés en phase 1.

### 8.4 Implication pour la plasticité

Le taux d'oubli est trop lent. L'homéostasie ($\rho = 0.0001$) ramène les conductances vers $C_0$ mais à un rythme de :

$$\Delta C_{ij}^{\text{homéo}} = 0.0001 \cdot (1.0 - C_{ij})$$

Pour une arête renforcée à $C_{ij} = 3.0$ : $\Delta C = -0.0002$ par tick. Il faudrait ~10 000 ticks d'homéostasie pure pour ramener cette arête à $C_0$. Avec la plasticité active (qui re-renforce partiellement), le re-routing est encore plus lent.

---

## 9. Résumé des Ajouts V5.1

| Concept | Formalisation | Section |
|---------|---------------|---------|
| Projection 3D→2D | Rotations $R_Y \cdot R_X$ + scale uniforme | §1 |
| Conductance moyenne par nœud | $\bar{C}_i = \frac{1}{\deg(i)} \sum C_{ij}$ | §2 |
| Arêtes top déviation | $\delta_{ij} = \|C_{ij} - C_0\|$, top-$K$ | §3 |
| Clusters co-renforcement | $\sum_k \mathbb{1}[i \in \text{path}_k] \geq 2$ | §4 |
| Accuracy sparkline | Fenêtre glissante 100 pts | §5 |
| Timeline conductance | $\bar{C}(t)$ échantillonné /10 éval | §6 |
| RandomBaseline (68.8%) | $C_{ij} \sim \mathcal{U}[0.1, 5.0]$, biais structurel | §7 |
| Remap (50.0%) | Re-routage à $T_{\text{remap}}$, homéostasie trop lente | §8 |

Aucune modification au modèle de base (tick pipeline, STDP, trois-facteurs, readout, reward). Les ajouts V5.1 sont strictement des **outils de diagnostic, de visualisation et de test**.
