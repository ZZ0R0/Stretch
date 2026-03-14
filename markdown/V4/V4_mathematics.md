# V4 — Modèle mathématique complet

> **Document exhaustif** de toutes les opérations mathématiques régissant le système Stretch V4.
> Extrait directement du code source (29 fichiers : 18 Rust + 11 WGSL).
>
> Le chemin GPU (11 shaders WGSL, single-submit par tick) est la référence principale.
> Le chemin CPU (Rust/rayon) est un fallback fonctionnellement identique.

---

## Table des matières

1. [Notations et conventions](#1-notations-et-conventions)
2. [Structures de données](#2-structures-de-données)
3. [Construction du graphe](#3-construction-du-graphe)
4. [Pipeline d'un tick](#4-pipeline-dun-tick)
5. [Phase 0 — Zones PID](#5-phase-0--zones-pid)
6. [Phase 1 — Injection de stimulus](#6-phase-1--injection-de-stimulus)
7. [Phase 2 — Contributions sources](#7-phase-2--contributions-sources)
8. [Phase 3 — Propagation CSR](#8-phase-3--propagation-csr)
9. [Phase 4 — Application des influences + Dissipation](#9-phase-4--application-des-influences--dissipation)
10. [Phase 5 — Plasticité : STDP, éligibilité, trois facteurs](#10-phase-5--plasticité--stdp-éligibilité-trois-facteurs)
11. [Phase 6 — Budget synaptique](#11-phase-6--budget-synaptique)
12. [Phase 7 — Synchronisation des conductances](#12-phase-7--synchronisation-des-conductances)
13. [Phase 8 — Readout de sortie](#13-phase-8--readout-de-sortie)
14. [Phase 9 — Récompense et dopamine](#14-phase-9--récompense-et-dopamine)
15. [Pacemakers](#15-pacemakers)
16. [Métriques](#16-métriques)
17. [Récapitulatif des paramètres](#17-récapitulatif-des-paramètres)
18. [Conditions de stabilité](#18-conditions-de-stabilité)

---

## 1. Notations et conventions

| Symbole | Signification | Domaine |
|---------|---------------|---------|
| $N$ | Nombre de nœuds | $\mathbb{N}$ |
| $E$ | Nombre d'arêtes (dirigées) | $\mathbb{N}$ |
| $a_i(t)$ | Activation du nœud $i$ au tick $t$ | $[0, 10]$ |
| $\theta_i$ | Seuil de base du nœud $i$ | $\mathbb{R}^+$ |
| $f_i$ | Fatigue du nœud $i$ | $[0, 10]$ |
| $h_i$ | Inhibition du nœud $i$ | $[0, 10]$ |
| $m_i$ | Trace mémoire du nœud $i$ | $[0, 100]$ |
| $\xi_i$ | Excitabilité du nœud $i$ | $\mathbb{R}^+$ |
| $\theta_i^{\text{mod}}$ | Modulation PID du seuil | $\mathbb{R}$ |
| $g_i^{\text{mod}}$ | Modulation PID du gain | $\mathbb{R}$ |
| $C_{ij}$ | Conductance de l'arête $i \to j$ | $[C_{\min}, C_{\max}]$ |
| $d_{ij}$ | Distance euclidienne $i \to j$ | $\mathbb{R}^+$ |
| $e_{ij}$ | Trace d'éligibilité | $[-e_{\max}, e_{\max}]$ |
| $w_{ij}$ | Poids du kernel spatial | $[0, 1]$ |
| $\sigma_i$ | Type : +1 (excitateur), $-g_I$ (inhibiteur) | $\{-g_I, +1\}$ |
| $D(t)$ | Dopamine totale = tonique + phasique | $\mathbb{R}$ |
| $r(t)$ | Récompense au tick $t$ | $[-1, +1]$ |

**Seuil effectif :**

$$\theta_i^{\text{eff}} = \max\!\left(\frac{\theta_i + f_i + h_i + \theta_i^{\text{mod}}}{\max(\xi_i,\; 0{,}01)},\; 0{,}05\right)$$

Un nœud est **actif** si $a_i(t) > \theta_i^{\text{eff}}$.

---

## 2. Structures de données

### Nœud (`GpuNode` — 48 octets)

| Champ | Type | Rôle |
|-------|------|------|
| `activation` | f32 | $a_i$ |
| `threshold` | f32 | $\theta_i$ |
| `fatigue` | f32 | $f_i$ |
| `memory_trace` | f32 | $m_i$ |
| `excitability` | f32 | $\xi_i$ |
| `inhibition` | f32 | $h_i$ |
| `threshold_mod` | f32 | $\theta_i^{\text{mod}}$ |
| `last_activation_tick` | i32 | Dernier tick actif (-1 = jamais) |
| `activation_count` | u32 | Compteur d'activations |
| `is_excitatory` | u32 | 1 = excitateur, 0 = inhibiteur |
| `gain_mod` | f32 | $g_i^{\text{mod}}$ |

### Arête (`GpuEdge` — 32 octets)

| Champ | Type | Rôle |
|-------|------|------|
| `from_node` | u32 | Nœud source |
| `to_node` | u32 | Nœud cible |
| `conductance` | f32 | $C_{ij}$ |
| `eligibility` | f32 | $e_{ij}$ |
| `consolidated` | u32 | 0/1 |
| `consolidation_counter` | u32 | Ticks consécutifs au-dessus du seuil |
| `distance` | f32 | $d_{ij}$ |

### Zone (`GpuZone` — 48 octets)

| Champ | Type | Rôle |
|-------|------|------|
| `target_activity` | f32 | Consigne $\bar{a}^*$ |
| `activity_sum` | f32 | Activité moyenne mesurée |
| `member_count` | u32 | Nombre de nœuds dans la zone |
| `error`, `integral`, `error_prev` | f32 | État PID |
| `output` | f32 | Sortie PID $u$ |
| `theta_mod`, `gain_mod` | f32 | Modulations envoyées aux nœuds |
| `stable_ticks`, `is_stable` | u32 | Skip C6 |

---

## 3. Construction du graphe

### 3.1 Positions 3D

Les $N$ nœuds sont placés uniformément dans un cube $[0, L]^3$ (PRNG ChaCha8, graine configurable) :

$$\mathbf{p}_i = (x_i, y_i, z_i),\quad x_i, y_i, z_i \sim \mathcal{U}(0, L)$$

### 3.2 Topologie knn_3d (défaut V4)

Un KD-tree index les positions. Pour chaque nœud $i$, on trouve les $k$ plus proches voisins et on crée des arêtes **bidirectionnelles** $(i \to j)$ et $(j \to i)$ avec déduplication canonique $(min, max)$.

$$d_{ij} = \|\mathbf{p}_i - \mathbf{p}_j\|_2 = \sqrt{(x_i - x_j)^2 + (y_i - y_j)^2 + (z_i - z_j)^2}$$

Le nombre d'arêtes résultant est $E \approx N \times k$ (légèrement inférieur dû à la déduplication).

### 3.3 Topologie radius_3d

Au lieu de $k$-NN, chaque nœud se connecte à tous les voisins dans un rayon $R$ :

$$j \in \mathcal{N}(i) \iff d_{ij} \leq R$$

### 3.4 Types neuronaux E/I

Chaque nœud est assigné inhibiteur avec probabilité $p_I$ (défaut 0,2), excitateur sinon. Le type est immutable après construction.

### 3.5 Kernel spatial pré-calculé

Les poids du kernel $w_{ij}$ sont calculés une seule fois et stockés dans le CSR :

- **Exponentiel** (défaut) : $w_{ij} = e^{-\lambda \cdot d_{ij}}$
- **Gaussien** : $w_{ij} = e^{-\frac{1}{2}(\lambda \cdot d_{ij})^2}$

avec $\lambda$ = `spatial_decay`.

### 3.6 Structures CSR

Le graphe est stocké en format Compressed Sparse Row (entrante et sortante) :
- **Incoming CSR** : pour chaque nœud cible $j$, liste les arêtes $(i \to j)$ — utilisé par la propagation.
- **Outgoing CSR** : pour chaque nœud source $i$, liste les arêtes $(i \to j)$ — utilisé par le budget synaptique.

### 3.7 Partitionnement en zones (Voronoï)

$K$ centres sont choisis à intervalles réguliers ($\text{step} = N / K$). Chaque nœud est assigné au centre le plus proche :

$$z_i = \arg\min_{c \in \{0, \ldots, K-1\}} \|\mathbf{p}_i - \mathbf{p}_{c}\|_2^2$$

---

## 4. Pipeline d'un tick

Le tick V4 exécute les phases suivantes **en séquence** (aucun parallélisme inter-phase) :

```
┌─ Phase 0 ─┐  ┌─ Phase 1 ─┐  ┌─ Phase 2 ─┐  ┌─ Phase 3 ─┐
│ Zones PID  │→ │ Injection  │→ │ Source     │→ │Propagation │→
│ (3 passes) │  │ stimulus   │  │ Contribs   │  │ CSR        │
└────────────┘  └────────────┘  └────────────┘  └────────────┘

┌─ Phase 4 ──────────┐  ┌─ Phase 5 ──────────┐  ┌─ Phase 6 ─┐
│ Apply influences    │→ │ Plasticité          │→ │ Budget     │→
│ + Dissipation fusée │  │ STDP+elig+3-factor  │  │ synaptique │
└─────────────────────┘  └─────────────────────┘  └────────────┘

┌─ Phase 7 ─┐  ┌─ Phase 8 ─┐  ┌─ Phase 9 ──────┐
│ Sync cond. │→ │ Readout    │→ │ Reward+Dopamine │
│ edges→CSR  │  │ (si trial) │  │ (CPU)           │
└────────────┘  └────────────┘  └─────────────────┘
```

Sur GPU, les phases 0–8 sont un **single command buffer submit** par tick. Les phases 9 restent sur CPU (décision légère, <1 μs).

---

## 5. Phase 0 — Zones PID

### 5.1 Mesure de l'activité (zone_measure)

Pour chaque zone $z$, l'activité moyenne est mesurée par accumulation atomique :

$$\bar{a}_z = \frac{1}{|\mathcal{M}_z|} \sum_{i \in \mathcal{M}_z} a_i$$

où $\mathcal{M}_z$ est l'ensemble des nœuds membres de la zone $z$.

*GPU : accumulation en virgule fixe (×10⁶ → u32 atomique).*

### 5.2 Contrôleur PID (zone_pid)

Pour chaque zone $z$ :

$$\varepsilon_z(t) = \bar{a}^*_z - \bar{a}_z(t)$$

$$I_z(t) = \text{clamp}\!\left(I_z(t{-}1) + \varepsilon_z(t),\; -I_{\max},\; I_{\max}\right)$$

$$D_z(t) = \varepsilon_z(t) - \varepsilon_z(t{-}1)$$

$$u_z(t) = \text{clamp}\!\left(K_p \cdot \varepsilon_z + K_i \cdot I_z + K_d \cdot D_z,\; -u_{\max},\; u_{\max}\right)$$

### 5.3 Mode indirect (V3/V4) — Application aux nœuds (zone_apply)

Le PID ne modifie pas directement l'activation. Il modifie le **seuil** et le **gain** :

$$\theta_i^{\text{mod}} = -k_\theta \cdot u_{z(i)}$$

$$g_i^{\text{mod}} = k_g \cdot u_{z(i)}$$

Si l'activité est trop basse ($\varepsilon > 0$, $u > 0$), le seuil diminue ($\theta^{\text{mod}} < 0$) et le gain augmente.

### 5.4 Stabilité C6

$$\text{Si } |\varepsilon_z| < 0{,}01 \text{ pendant 50 ticks consécutifs} \Rightarrow \text{zone skippée}$$
$$\text{Réactivation si } |\varepsilon_z| > 0{,}05 \text{ (mesuré toutes les 10 ticks)}$$

---

## 6. Phase 1 — Injection de stimulus

### 6.1 Protocole de trials V4

L'entraînement est organisé en trials :
- **Warmup** : 20 ticks sans stimulus
- **Présentation** : 5 ticks d'injection
- **Délai** : `read_delay` ticks de propagation libre
- **Gap inter-trial** : 15 ticks
- **Période totale** : $5 + \text{read\_delay} + 15 + 1$ ticks par trial

### 6.2 Groupes d'entrée spatiaux

Les groupes d'entrée sont sélectionnés par proximité spatiale dans le cube 3D :
- **Classe 0** : $G_{\text{group\_size}}$ nœuds les plus proches de $(0, L/2, L/2)$
- **Classe 1** : $G_{\text{group\_size}}$ nœuds les plus proches de $(L, L/2, L/2)$

(Positions diamétralement opposées pour maximiser la séparation spatiale.)

### 6.3 Injection (shader injection.wgsl)

Au début de chaque trial, toutes les activations sont remises à zéro :

$$a_i(t) \leftarrow 0 \quad \forall i$$

Pendant les ticks de présentation :

$$a_i(t) \leftarrow \min(a_i(t) + I_{\text{stim}},\; 10) \quad \forall i \in G_{\text{class}}$$

avec $I_{\text{stim}}$ = `stimulus_intensity` (défaut 1,5).

### 6.4 Pacemakers (mode legacy)

$$a_i(t) \leftarrow a_i(t) + A \cdot \sin(2\pi f t + \varphi) + \text{offset}$$

Borné à $[0, 10]$.

---

## 7. Phase 2 — Contributions sources

### Shader `source_contribs.wgsl` — 1 thread par nœud

Pour chaque nœud $i$ :

$$s_i = \begin{cases}
a_i \cdot \sigma_i \cdot (1 + g_i^{\text{mod}}) \cdot G & \text{si } a_i > \theta_i^{\text{eff}} \\
0 & \text{sinon}
\end{cases}$$

où :
- $\sigma_i = +1$ si excitateur, $-g_I$ si inhibiteur
- $G$ = `propagation_gain` (gain global)
- $g_I$ = `gain_inhibitory` (facteur d'amplification inhibitrice)
- $g_i^{\text{mod}}$ = modulation de gain par la zone PID

**Seuls les nœuds actifs contribuent** (gate par seuil effectif).

---

## 8. Phase 3 — Propagation CSR

### Shader `propagation.wgsl` — 1 thread par nœud cible

Pour chaque nœud cible $j$, on accumule les influences de toutes les sources entrantes :

$$I_j = \sum_{i \in \mathcal{N}_{\text{in}}(j)} s_i \cdot C_{ij} \cdot w_{ij}$$

où :
- $s_i$ = contribution source (phase 2)
- $C_{ij}$ = conductance de l'arête $i \to j$ (dans l'ordre CSR)
- $w_{ij}$ = poids du kernel spatial pré-calculé : $e^{-\lambda d_{ij}}$

La boucle interne parcourt le CSR `[offsets[j] .. offsets[j+1])` en sautant les sources nulles ($s_i = 0$).

### CPU : mode adaptatif

Le chemin CPU a une optimisation supplémentaire :
- Si < 30% des nœuds sont actifs → **outgoing CSR** (itère depuis les sources actives)
- Sinon → **incoming CSR** (itère par cible, comme le GPU)

---

## 9. Phase 4 — Application des influences + Dissipation

### Shader `apply_and_dissipate.wgsl` — 1 thread par nœud (fusionné)

#### 9.1 Application de l'influence

$$a_i(t) \leftarrow \text{clamp}(a_i(t) + I_i,\; 0,\; 10)$$

#### 9.2 Détection d'activité

Si $a_i > \theta_i^{\text{eff}}$ après ajout de l'influence :
- `last_activation_tick` ← tick courant
- `activation_count` += 1

#### 9.3 Fatigue

$$f_i(t) \leftarrow \begin{cases}
f_i(t) + \alpha_f \cdot a_i(t) & \text{si actif} \\
f_i(t) & \text{sinon}
\end{cases}$$

$$f_i(t) \leftarrow \text{clamp}\!\left(f_i(t) \cdot (1 - \beta_f),\; 0,\; 10\right)$$

avec $\alpha_f$ = `fatigue_gain`, $\beta_f$ = `fatigue_recovery`.

#### 9.4 Inhibition

$$h_i(t) \leftarrow \begin{cases}
h_i(t) + \alpha_h & \text{si actif} \\
h_i(t) & \text{sinon}
\end{cases}$$

$$h_i(t) \leftarrow \text{clamp}\!\left(h_i(t) \cdot (1 - \beta_h),\; 0,\; 10\right)$$

avec $\alpha_h$ = `inhibition_gain`, $\beta_h$ = `inhibition_decay`.

#### 9.5 Trace mémoire

$$m_i(t) \leftarrow \begin{cases}
m_i(t) + \alpha_m \cdot a_i(t) & \text{si actif} \\
m_i(t) & \text{sinon}
\end{cases}$$

$$m_i(t) \leftarrow \text{clamp}\!\left(m_i(t) \cdot (1 - \beta_m),\; 0,\; 100\right)$$

avec $\alpha_m$ = `trace_gain`, $\beta_m$ = `trace_decay`.

#### 9.6 Excitabilité ← trace mémoire

$$\xi_i = 1 + 0{,}1 \cdot \min(m_i,\; 5)$$

L'excitabilité croît linéairement avec la trace mémoire, plafonnée à $\xi_{\max} = 1{,}5$.

#### 9.7 Décroissance de l'activation

$$a_i(t) \leftarrow \max\!\left(a_i(t) \cdot (1 - \delta_{\text{eff}}),\; a_{\min}\right)$$

avec :

$$\delta_{\text{eff}} = \text{clamp}\!\left(\delta \cdot (1 + j_i),\; 0,\; 1\right)$$

où $\delta$ = `activation_decay`, et $j_i$ est un jitter déterministe par nœud :

$$j_i = \left(\frac{h_i \gg 16}{65536} - 0{,}5\right) \cdot 2 \cdot \delta_j$$

$h_i = \text{hash}(\text{idx}, \text{tick})$ est un entier pseudo-aléatoire reproductible.

---

## 10. Phase 5 — Plasticité : STDP, éligibilité, trois facteurs

### Shader `plasticity.wgsl` — 1 thread par arête

C'est le cœur de l'apprentissage V4. Cinq étapes séquentielles par arête :

### 10.1 STDP — direction $\psi_{ij}$

$$\psi_{ij} = \begin{cases}
A^+ \cdot e^{-\Delta t / \tau^+} & \text{si } \Delta t > 0 \text{ (pré avant post, LTP)} \\
-A^- \cdot e^{\Delta t / \tau^-} & \text{si } \Delta t < 0 \text{ (post avant pré, LTD)} \\
0 & \text{si } \Delta t = 0 \text{ ou un endpoint n'a jamais été actif}
\end{cases}$$

avec $\Delta t = t_{\text{post}} - t_{\text{pré}}$.

**Important :** $\psi$ ne modifie **pas** la conductance. Il alimente uniquement la trace d'éligibilité.

### 10.2 Trace d'éligibilité

$$e_{ij}(t) = \text{clamp}\!\left(\gamma_e \cdot e_{ij}(t{-}1) + \psi_{ij},\; -e_{\max},\; e_{\max}\right)$$

avec $\gamma_e$ = `elig_decay` (défaut 0,95), $e_{\max}$ = `elig_max` (défaut 5,0).

La trace d'éligibilité sert de **mémoire temporaire** : elle retient la direction STDP pendant que le signal de récompense arrive (potentiellement décalé de plusieurs ticks).

### 10.3 Règle des trois facteurs (Frémaux & Gerstner 2016)

**Si $|e_{ij}| > 10^{-6}$ (seuil « hot edge ») :**

$$\Delta C_{ij} = \eta \cdot \delta_d \cdot e_{ij}$$

$$C_{ij}(t) = \text{clamp}\!\left(C_{ij}(t{-}1) + \Delta C_{ij},\; C_{\min},\; C_{\max}\right)$$

où :
- $\eta$ = `plasticity_gain` (taux d'apprentissage trois-facteurs)
- $\delta_d$ = signal dopaminergique :

$$\delta_d = \begin{cases}
D_{\text{phasique}} \cdot e^{-\lambda_s \cdot d(\mathbf{p}_j, \mathbf{p}_{\text{reward}})} & \text{si spatial activé} \\
D_{\text{phasique}} & \text{sinon (global)}
\end{cases}$$

Le produit $\delta_d \cdot e_{ij}$ assure que :
- Récompense positive + éligibilité positive → **renforcement** (LTP confirmée)
- Récompense positive + éligibilité négative → **affaiblissement** (LTD confirmée)
- Récompense négative → directions inversées (anti-Hebb)
- Pas de récompense ($\delta_d \approx 0$) → **pas de changement** (indifférence)

### 10.4 Décroissance homéostatique

Pour les arêtes **non consolidées** :

$$C_{ij}(t) \leftarrow C_{ij}(t) + \rho \cdot (C_0 - C_{ij}(t))$$

$$C_{ij}(t) = \text{clamp}(C_{ij}(t),\; C_{\min},\; C_{\max})$$

avec $\rho$ = `homeostatic_rate` (=`decay`, défaut 0,0001) et $C_0$ = `baseline_cond` (=`conductance`, défaut 1,0).

Ce terme ramène lentement toutes les conductances vers la baseline, empêchant la dérive à long terme.

### 10.5 Consolidation mémoire

**Conditions :**
1. $D(t) > D_{\text{consol}}$ (dopamine au-dessus du seuil de consolidation)
2. $e_{ij} > 0$ (éligibilité positive)
3. $C_{ij} \geq C_{\text{consol}}$ (conductance au-dessus du seuil, défaut 4,0)

Si les 3 conditions sont réunies :

$$\text{counter}_{ij} \leftarrow \text{counter}_{ij} + 1$$

$$\text{Si counter}_{ij} \geq T_{\text{consol}} \Rightarrow \text{consolidated}_{ij} = \text{true}$$

avec $T_{\text{consol}}$ = `ticks_required` (défaut 50).

**Effet de la consolidation :** l'arête consolidée est **protégée** de la décroissance homéostatique et du budget synaptique. Sa conductance ne peut plus diminuer (sauf par la règle des trois facteurs elle-même).

---

## 11. Phase 6 — Budget synaptique

### Shader `budget.wgsl` — 2 passes

Le budget synaptique normalise les conductances sortantes de chaque nœud source.

#### Pass 1 — Sommation atomique (`budget_sum`)

Pour chaque nœud source $i$ :

$$T_i = \sum_{j \in \mathcal{N}_{\text{out}}(i)} C_{ij}$$

*GPU : accumulation en virgule fixe (×10⁶ → u32 atomique).*

#### Pass 2 — Normalisation (`budget_scale`)

Si $T_i > B$ (budget dépassé) :

$$C_{ij} \leftarrow \text{clamp}\!\left(C_{ij} \cdot \frac{B}{T_i},\; C_{\min},\; C_{\max}\right) \quad \forall j \in \mathcal{N}_{\text{out}}(i),\; \text{si non consolidée}$$

Les arêtes consolidées sont **exemptées** du rescaling.

**CPU :** utilise un suivi incrémental (C4) — seuls les nœuds sources dont au moins une arête a changé sont vérifiés.

---

## 12. Phase 7 — Synchronisation des conductances

### Shader `sync_conductances.wgsl` — 1 thread par entrée CSR

La plasticité modifie les conductances dans le buffer `edges[]` indexé par arête. Mais la propagation (phase 3) lit les conductances dans l'ordre CSR. Il faut donc recopier :

$$\text{conductances}[k] \leftarrow \text{edges}[\text{csr\_edge\_indices}[k]].\text{conductance}$$

Ce shader remplace le round-trip CPU (download edges → reorder → upload).

---

## 13. Phase 8 — Readout de sortie

### Shader `readout.wgsl` — 1 thread par nœud de sortie

Les groupes de sortie sont sélectionnés spatialement (comme les entrées) :
- **Classe 0** : $G$ nœuds proches de $(L \times 0{,}25,\; L/2,\; L/2)$
- **Classe 1** : $G$ nœuds proches de $(L \times 0{,}75,\; L/2,\; L/2)$

Le score de chaque classe est la **somme des activations** :

$$S_c = \sum_{i \in G_c^{\text{out}}} a_i$$

*GPU : accumulation atomique en virgule fixe (×10⁶).*

La décision est l'**argmax** :

$$\hat{y} = \arg\max_c\; S_c$$

La **marge** est la différence entre les deux meilleurs scores :

$$\text{marge} = S_{\hat{y}} - \max_{c \neq \hat{y}} S_c$$

---

## 14. Phase 9 — Récompense et dopamine

Cette phase reste sur CPU (une seule opération par trial, <1 μs).

### 14.1 Attribution du reward

$$r(t) = \begin{cases}
r^+ & \text{si } \hat{y} = y^* \text{ (correct)} \\
r^- & \text{si } \hat{y} \neq y^* \text{ (incorrect)}
\end{cases}$$

avec $r^+ = +1$ et $r^- = -1$ par défaut.

### 14.2 Dynamique dopaminergique

$$D_{\text{phasique}}(t+1) = (1 - \beta_D) \cdot D_{\text{phasique}}(t) + \alpha_D \cdot r(t)$$

$$D_{\text{phasique}}(t) = \text{clamp}\left(D_{\text{phasique}},\; -D_{\max},\; D_{\max}\right)$$

$$D(t) = D_{\text{tonique}} + D_{\text{phasique}}(t)$$

avec :
- $\beta_D$ = `phasic_decay` (défaut 0,15)
- $\alpha_D$ = `reward_gain` (défaut 1,0)
- $D_{\text{tonique}}$ = `tonic` (défaut 0,1)
- $D_{\max}$ = `phasic_max` (défaut 2,0)

La dopamine **décroît aussi entre les trials** (à chaque tick, `update(0.0)` est appelé).

### 14.3 Centre de récompense spatial

Si `spatial_lambda` > 0, un centre de récompense est calculé :
- **Si correct :** barycentre du groupe de sortie de la classe cible
- **Si incorrect :** barycentre du groupe de sortie de la classe prédite

$$\mathbf{p}_{\text{reward}} = \frac{1}{|G_c|} \sum_{i \in G_c} \mathbf{p}_i$$

La dopamine locale est ensuite :

$$\delta_{d,\text{local}}(j) = D_{\text{phasique}} \cdot e^{-\lambda_s \cdot \|\mathbf{p}_j - \mathbf{p}_{\text{reward}}\|}$$

Ce vecteur est pré-calculé et uploadé sur GPU quand le centre change.

---

## 15. Pacemakers

Oscillateurs sinusoïdaux injectés dans des nœuds spécifiques (mode V2, optionnel) :

$$a_i(t) \leftarrow a_i(t) + A \sin(2\pi f t + \varphi) + \text{offset}$$

Borné à $[0, 10]$. Les pacemakers sont appliqués **avant** la propagation.

---

## 16. Métriques

### Shader `metrics_reduce.wgsl` — 2 entry points

#### reduce_nodes (1 thread par nœud)

| Métrique | Formule |
|----------|---------|
| `active_count` | $\sum_i \mathbb{1}[a_i > \theta_i^{\text{eff}}]$ |
| `global_energy` | $\sum_i a_i$ |
| `max_activation` | $\max_i a_i$ |
| `mean_memory_trace` | $\frac{1}{N}\sum_i m_i$ |
| `max_memory_trace` | $\max_i m_i$ |
| `mean_fatigue` | $\frac{1}{N}\sum_i f_i$ |
| `active_excitatory` | $\sum_i \mathbb{1}[\text{exc}_i \wedge \text{actif}_i]$ |
| `active_inhibitory` | $\sum_i \mathbb{1}[\text{inh}_i \wedge \text{actif}_i]$ |
| `excitatory_energy` | $\sum_{i:\text{exc}} a_i$ |
| `inhibitory_energy` | $\sum_{i:\text{inh}} a_i$ |

*GPU : sommes en virgule fixe (×10³), max via bitcast trick (pour f32 ≥ 0, l'ordre u32 est préservé).*

#### reduce_edges (1 thread par arête)

| Métrique | Formule |
|----------|---------|
| `mean_conductance` | $\frac{1}{E}\sum_{ij} C_{ij}$ |
| `max_conductance` | $\max_{ij} C_{ij}$ |
| `consolidated_count` | $\sum_{ij} \mathbb{1}[\text{consolidated}_{ij}]$ |
| `mean_eligibility` | $\frac{1}{E}\sum_{ij} |e_{ij}|$ |

---

## 17. Récapitulatif des paramètres

### Domaine

| Paramètre | Config | Défaut V4 | Rôle |
|-----------|--------|-----------|------|
| $N$ | `domain.size` | 50 000 | Nombre de nœuds |
| $k$ | `domain.k_neighbors` | 10 | Voisins kNN |
| $L$ | `domain.domain_extent` | 100,0 | Taille du cube 3D |
| $p_I$ | `neuron_types.inhibitory_fraction` | 0,2 | Fraction inhibitrice |

### Nœuds

| Paramètre | Config | Défaut | Rôle |
|-----------|--------|--------|------|
| $\theta$ | `node_defaults.threshold` | 0,2 | Seuil de base |
| $\xi_0$ | `node_defaults.excitability` | 1,0 | Excitabilité initiale |

### Propagation

| Paramètre | Config | Défaut V4 | Rôle |
|-----------|--------|-----------|------|
| $G$ | `propagation.gain` | 0,8 | Gain de propagation global |
| $g_I$ | `propagation.gain_inhibitory` | 0,8 | Facteur inhibiteur |
| $\lambda$ | `propagation.spatial_decay` | 0,3 | Atténuation spatiale |
| kernel | `propagation.kernel` | exponential | Type de kernel |

### Dissipation

| Paramètre | Config | Défaut V4 | Rôle |
|-----------|--------|-----------|------|
| $\delta$ | `dissipation.activation_decay` | 0,25 | Taux de décroissance |
| $a_{\min}$ | `dissipation.activation_min` | 0,01 | Potentiel de repos |
| $\alpha_f$ | `dissipation.fatigue_gain` | 0,20 | Gain de fatigue |
| $\beta_f$ | `dissipation.fatigue_recovery` | 0,05 | Récupération fatigue |
| $\alpha_h$ | `dissipation.inhibition_gain` | 0,12 | Gain d'inhibition |
| $\beta_h$ | `dissipation.inhibition_decay` | 0,03 | Décroissance inhibition |
| $\alpha_m$ | `dissipation.trace_gain` | 0,1 | Gain trace mémoire |
| $\beta_m$ | `dissipation.trace_decay` | 0,005 | Décroissance trace |
| $\delta_j$ | `dissipation.decay_jitter` | 0,15 | Amplitude jitter (±15%) |

### STDP

| Paramètre | Config | Défaut V4 | Rôle |
|-----------|--------|-----------|------|
| $A^+$ | `stdp.a_plus` | 0,005 | Amplitude LTP |
| $A^-$ | `stdp.a_minus` | 0,005 | Amplitude LTD |
| $\tau^+$ | `stdp.tau_plus` | 20 | Constante de temps LTP (ticks) |
| $\tau^-$ | `stdp.tau_minus` | 20 | Constante de temps LTD (ticks) |

### Éligibilité

| Paramètre | Config | Défaut V4 | Rôle |
|-----------|--------|-----------|------|
| $\gamma_e$ | `eligibility.decay` | 0,95 | Décroissance par tick |
| $e_{\max}$ | `eligibility.max` | 5,0 | Plafond |

### Dopamine

| Paramètre | Config | Défaut V4 | Rôle |
|-----------|--------|-----------|------|
| $D_{\text{tonique}}$ | `dopamine.tonic` | 0,1 | Niveau de base |
| $\beta_D$ | `dopamine.phasic_decay` | 0,15 | Décroissance phasique |
| $\alpha_D$ | `dopamine.reward_gain` | 1,0 | Reward → dopamine |
| $\eta$ | `dopamine.plasticity_gain` | 2,0 | Taux d'apprentissage 3-facteurs |
| $D_{\text{consol}}$ | `dopamine.consolidation_threshold` | 0,3 | Gating consolidation |
| $D_{\max}$ | `dopamine.phasic_max` | 2,0 | Borne phasique |
| $\lambda_s$ | `dopamine.spatial_lambda` | 0,0 | Décroissance spatiale (0 = global) |

### Conductances / Plasticité

| Paramètre | Config | Défaut V4 | Rôle |
|-----------|--------|-----------|------|
| $C_0$ | `edge_defaults.conductance` | 1,0 | Conductance de base (= baseline homéostatique) |
| $C_{\min}$ | `edge_defaults.conductance_min` | 0,1 | Borne inférieure |
| $C_{\max}$ | `edge_defaults.conductance_max` | 5,0 | Borne supérieure |
| $\rho$ | `edge_defaults.decay` | 0,0001 | Taux homéostatique |

### Consolidation

| Paramètre | Config | Défaut V4 | Rôle |
|-----------|--------|-----------|------|
| $C_{\text{consol}}$ | `consolidation.threshold` | 4,0 | Seuil de conductance pour consolider |
| $T_{\text{consol}}$ | `consolidation.ticks_required` | 50 | Ticks consécutifs nécessaires |

### Budget synaptique

| Paramètre | Config | Défaut V4 | Rôle |
|-----------|--------|-----------|------|
| $B$ | `synaptic_budget.budget` | 30,0 | Budget max par nœud source |

### Zones PID

| Paramètre | Config | Défaut V4 | Rôle |
|-----------|--------|-----------|------|
| $K$ | `zones.num_zones` | 8 | Nombre de zones |
| $\bar{a}^*$ | `zones.target_activity` | 0,3 | Consigne d'activité |
| $K_p$ | `zones.kp` | 0,5 | Gain proportionnel |
| $K_i$ | `zones.ki` | 0,05 | Gain intégral |
| $K_d$ | `zones.kd` | 0,1 | Gain dérivé |
| $u_{\max}$ | `zones.pid_output_max` | 2,0 | Borne sortie PID |
| $I_{\max}$ | `zones.pid_integral_max` | 5,0 | Anti-windup intégral |
| $k_\theta$ | `zones.k_theta` | 0,3 | Couplage PID → seuil |
| $k_g$ | `zones.k_gain` | 0,2 | Couplage PID → gain |

### I/O

| Paramètre | Config | Défaut V4 | Rôle |
|-----------|--------|-----------|------|
| `num_classes` | `input.num_classes` | 2 | Nombre de classes |
| `group_size` | `input.group_size` | 50 | Nœuds par groupe I/O |
| $I_{\text{stim}}$ | `input.intensity` | 1,5 | Intensité d'injection |
| `read_delay` | `output.read_delay` | 10 | Latence avant readout |

### Récompense

| Paramètre | Config | Défaut V4 | Rôle |
|-----------|--------|-----------|------|
| $r^+$ | `reward.reward_positive` | 1,0 | Reward correct |
| $r^-$ | `reward.reward_negative` | -1,0 | Reward incorrect |

---

## 18. Conditions de stabilité

### 18.1 Condition de propagation sous-critique

Pour que le réseau ne diverge pas, l'influence totale reçue par un nœud ne doit pas dépasser sa capacité d'absorption. Avec un nœud à conductance uniforme $C$ et $k$ voisins, le gain effectif est :

$$G_{\text{eff}} = G \cdot C \cdot \bar{w} \cdot k$$

où $\bar{w}$ est le kernel moyen. La propagation est **sous-critique** si :

$$G_{\text{eff}} < \frac{\theta}{a_{\text{typical}}}$$

Avec les défauts V4 ($G = 0{,}8$, $C = 1{,}0$, $k = 10$, $\lambda = 0{,}3$), le kernel moyen exponentiel donne $\bar{w} \approx 0{,}18$, soit $G_{\text{eff}} \approx 1{,}44$, ce qui est juste au-dessus du seuil — l'activité persiste mais ne diverge pas grâce à la fatigue et à l'inhibition.

### 18.2 Balance E/I

Avec 80% excitateurs et 20% inhibiteurs à gain identique ($g_I = 0{,}8$) :

$$\frac{E_{\text{inh}}}{E_{\text{exc}}} = \frac{0{,}2 \times 0{,}8}{0{,}8 \times 1{,}0} = 0{,}20$$

Soit 20% d'énergie inhibitrice — suffisant pour éviter l'emballement tout en permettant la propagation.

### 18.3 Budget synaptique

Le budget $B = 30$ avec $k = 10$ voisins sortants donne une conductance moyenne maximale de $\bar{C}_{\max} = B / k = 3{,}0$. Les arêtes consolidées (jusqu'à $C_{\max} = 5{,}0$) sont exemptées, mais la normalisation post-budget empêche l'inflation globale.

### 18.4 Demi-vie de l'éligibilité

La trace d'éligibilité décroît exponentiellement :

$$e(t + \Delta t) = \gamma_e^{\Delta t} \cdot e(t)$$

La demi-vie est :

$$t_{1/2} = \frac{\ln 2}{\ln(1/\gamma_e)} = \frac{0{,}693}{0{,}0513} \approx 13{,}5 \text{ ticks}$$

Avec un `read_delay` de 10 ticks, le signal de récompense arrive quand l'éligibilité conserve encore $\gamma_e^{10} = 0{,}95^{10} \approx 60\%$ de sa valeur initiale.

### 18.5 Convergence homéostatique

La décroissance homéostatique $C \leftarrow C + \rho(C_0 - C)$ a un temps caractéristique de :

$$\tau_{\text{homeo}} = \frac{1}{\rho} = \frac{1}{0{,}0001} = 10\,000 \text{ ticks}$$

Ce qui est très lent (200× la durée d'un trial), permettant à la plasticité reward de s'exprimer pleinement avant que l'homéostasie ne ramène les conductances vers la baseline.
