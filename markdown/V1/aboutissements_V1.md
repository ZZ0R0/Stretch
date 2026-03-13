# Aboutissements V1

## Résumé

La V1 accomplit la transition d'un substrat 2D sur grille fixe (V0, 400 nœuds) vers un **système spatial 3D à topologie non-grid**, scalable à 50 000 nœuds. La formalisation mathématique complète du système a été réalisée, incluant l'analyse des points d'équilibre et des régimes dynamiques.

---

## 1. Substrat spatial 3D non-grid

### Réalisation
- Les nœuds sont placés **aléatoirement** dans un cube $[0, L]^3$ (domaine continu).
- La grille 2D rigide de V0 est abandonnée au profit d'une géométrie stochastique réaliste.
- Les positions sont générées par un PRNG déterministe (ChaCha8, seed configurable), garantissant la **reproductibilité**.

### Topologies implémentées
| Topologie | Méthode | Complexité |
|---|---|---|
| `grid2d` | Grille régulière (rétro-compatible V0) | $O(N)$ |
| `random_sparse` | Connexions aléatoires | $O(N)$ |
| `knn_3d` | K plus proches voisins via KD-tree | $O(N \log N)$ |
| `radius_3d` | Tous les voisins dans un rayon $r$ | $O(N \log N)$ |

### Indexation spatiale
- Intégration du crate **kiddo 4** (KD-tree 3D).
- Requêtes KNN et radius en $O(\log N)$ par nœud.
- Construction de l'index en $O(N \log N)$.

### Validation
- **10 000 nœuds** : construction du graphe + 500 ticks en ~2s.
- **50 000 nœuds** : 1000 ticks en ~13,1s (release).
- Topologies KNN et radius produisent des dynamiques distinctes et cohérentes.

---

## 2. Calibration des paramètres 3D

### Problème résolu
Les distances euclidiennes moyennes en 3D (~18 unités pour 10k nœuds dans un cube 100³) sont très supérieures aux distances sur grille 2D (~1-2). Les paramètres V0 produisaient des ondes mortes en 3D.

### Solution
Re-calibration complète :
- **Gain** : $0{,}4 \to 0{,}6$ (compense les distances plus grandes)
- **Spatial decay** ($\lambda$) : $0{,}8 \to 0{,}15$ (noyau exponentiel plus large)
- **Seuil** ($\theta$) : $0{,}3 \to 0{,}2$ (facilite la propagation)
- **Fatigue gain** ($\gamma_f$) : $0{,}15 \to 0{,}20$ (compense le gain accru)
- **Inhibition gain** ($\gamma_h$) : $0{,}08 \to 0{,}12$ (idem)

### Résultat
Le facteur d'amplification moyen par saut : $\mu \approx 1{,}21$ — dans la zone de propagation contrôlée ($\mu > 1$ mais stabilisé par les rétroactions).

---

## 3. Potentiel de repos et brisure de symétrie

### Problème résolu (flatline)
En V0, l'activation décroissait vers exactement 0,0 partout, créant un **plancher mort** : énergie globale = 0, aucune dynamique résiduelle, simulation plate.

### Solutions implémentées

#### 3a. Activation minimale ($a_{\min}$)
- Chaque nœud conserve un **potentiel de repos** configurable (défaut : 0,01).
- Équation : $a_i(t+1) = \max(a_i(t) \times (1-\alpha),\; a_{\min})$
- Énergie plancher : $E^* = N \times a_{\min}$ (ex. 100 pour 10k nœuds).

#### 3b. Jitter stochastique sur le decay ($J$)
- Le taux de decay est perturbé individuellement pour chaque nœud à chaque tick.
- $\alpha_i^{\text{eff}}(t) = \alpha \times (1 + \xi_i),\quad \xi_i \sim \mathcal{U}(-J, +J)$
- Brise la symétrie artificielle du réseau homogène.
- Crée des micro-variations d'activation autour du repos.
- Désactivable ($J = 0$) pour retrouver le comportement déterministe V0.

### Impact
- Plus de flatline : le système maintient un fond d'activité sub-seuil.
- Compatible avec les analyses d'équilibre (le repos $a^* = a_{\min}$ reste le seul point fixe stable).

---

## 4. Visualisation 3D

### Réalisation
- **Projection orthographique** des positions 3D vers l'écran 2D.
- Rotation interactive : **WASD** pour orbiter autour du centre de masse.
- Mode adaptatif : grille pour `grid2d`, nuage de points pour KNN/radius.
- Code couleur thermique : bleu (repos) → rouge (excité) → blanc (saturé).

### Interface
- Panneau latéral : tick courant, nœuds actifs, énergie globale, fatigue/inhibition/trace moyennes.
- Sparkline d'énergie sur les 120 derniers ticks.
- Zoom automatique au domaine.

---

## 5. Formalisation mathématique complète

### Contenu de `maths.md`
- **15 sections** couvrant l'intégralité du modèle.
- Variables d'état (6 par nœud, 4 par arête), 16 paramètres globaux.
- Séquence complète d'un tick en 4 phases avec équations exactes (E1–E10).
- Quantités dérivées : seuil effectif $\theta^{\text{eff}}$, prédicat d'activation $\sigma_i$.

### Analyse d'équilibre
- **Point de repos** unique et stable : $\mathbf{x}^* = (a_{\min}, 0, 0, 0, 1, w_{\min}, 0)$.
- Condition de stabilité : $a_{\min} < \theta$ (vérifié : $0{,}01 < 0{,}2$).
- **Pas d'équilibre actif stable** : démontré par analyse des rétroactions (fatigue + inhibition → seuil effectif monte à ~2,8+ en saturation).
- Demi-vies calculées pour chaque variable (activation ~2,4 ticks, trace mémoire ~138 ticks, conductance ~250 ticks).

### Condition de non-saturation
$$\mu = \bar{k} \cdot \bar{w} \cdot K(\bar{d}) \cdot g \cdot \frac{1-\alpha}{\alpha} < \mu_{\text{crit}}$$

Calculé : $\mu \approx 1{,}21$ pour les paramètres V1 calibrés.

---

## 6. Plasticité Hebbienne

### Fonctionnement validé (hérité de V0, confirmé en 3D)
- Règle de co-activation avec seuil ($c_\theta = 0{,}2$).
- Renforcement supra-seuil : $\Delta w = +\eta_+ \cdot p \cdot (c - c_\theta)$.
- Affaiblissement sub-seuil : $\Delta w = -\eta_- \cdot p$.
- Conductance bornée $[w_{\min}, w_{\max}] = [0{,}1;\; 5{,}0]$.

### Résultat d'apprentissage en 3D
- Ratio d'apprentissage mesuré : **2,54×** (nœuds actifs post-entraînement vs. pré-entraînement).
- Les trajectoires renforcées émergent dans l'espace 3D continu.
- La mémoire structurelle (conductances + traces) persiste ~200 ticks après l'entraînement.

---

## 7. Architecture logicielle

### Cargo workspace
```
Stretch/
├── stretch-core/    # Bibliothèque (lib) — moteur de simulation
│   └── src/
│       ├── config.rs        # Serde structs (TOML)
│       ├── node.rs          # État + dynamiques nœud
│       ├── edge.rs          # État + plasticité arête
│       ├── domain.rs        # Graphe + topologie + KD-tree
│       ├── propagation.rs   # Noyaux + influences
│       ├── dissipation.rs   # (intégré dans node.rs)
│       ├── plasticity.rs    # Règle Hebbienne
│       ├── stimulus.rs      # Injection externe
│       ├── simulation.rs    # Orchestration + RNG
│       └── metrics.rs       # Instrumentation
├── stretch-cli/     # Binaire CLI (headless)
└── stretch-viz/     # Binaire visualisation (macroquad)
```

### Qualités
- **Déterminisme** : ChaCha8 seedé, résultats reproductibles.
- **Configuration externe** : TOML, aucun paramètre hard-codé.
- **Modularité** : core séparé des frontends (CLI / viz).
- **Performance** : release-mode, 50k nœuds en ~13ms/tick.

---

## 8. Configurations validées

| Config | Nœuds | Topologie | Ticks | Usage |
|---|---|---|---|---|
| `config.toml` | 400 | grid2d | 500 | Rétro-compat V0 |
| `config_v1_knn.toml` | 10k | knn_3d (K=10) | 500 | Référence V1 |
| `config_v1_radius.toml` | 10k | radius_3d (r=8) | 500 | Variante topologique |
| `config_v1_training.toml` | 10k | knn_3d | 1000 | Apprentissage 3 phases |
| `config_v1_benchmark_50k.toml` | 50k | knn_3d | 1000 | Stress-test perf |

---

## 9. Résumé des objectifs V1

| Objectif | Statut |
|---|---|
| Passage au 3D non-grid | ✅ |
| Indexation spatiale KD-tree | ✅ |
| Topologies KNN et radius | ✅ |
| Calibration paramètres 3D | ✅ |
| Résolution du flatline (repos / jitter) | ✅ |
| Visualisation 3D interactive | ✅ |
| Formalisation mathématique complète | ✅ |
| Analyse d'équilibre | ✅ |
| Scalabilité 50k nœuds | ✅ |
| Apprentissage Hebbien en 3D | ✅ |
| Ondes sinusoïdales / oscillations entretenues | ❌ (V2) |
| Neurones de contrôle / régulateurs PID | ❌ (V2) |
