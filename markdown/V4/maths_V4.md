# Stretch — Référence mathématique complète (V1→V4)

> Extraction exhaustive de **toutes** les équations implémentées dans le code,
> avec analyse numérique des plages de valeurs et diagnostic de la sur-stabilisation V4.

---

## Table des matières

1. [Propagation du signal](#1-propagation-du-signal)
2. [Dynamique des nœuds](#2-dynamique-des-nœuds)
3. [Dynamique des arêtes — Plasticité hebbienne](#3-dynamique-des-arêtes--plasticité-hebbienne)
4. [STDP (Spike-Timing-Dependent Plasticity)](#4-stdp)
5. [Traces d'éligibilité (V4)](#5-traces-déligibilité-v4)
6. [Dopamine (V4)](#6-dopamine-v4)
7. [Modulation reward → conductance (V4)](#7-modulation-reward--conductance-v4)
8. [Budget synaptique](#8-budget-synaptique)
9. [Consolidation](#9-consolidation)
10. [Contrôle PID par zones](#10-contrôle-pid-par-zones)
11. [Pacemakers](#11-pacemakers)
12. [Entrée / Sortie (V4)](#12-entrée--sortie-v4)
13. [Diagnostic : Pourquoi la V4 sur-stabilise](#13-diagnostic--pourquoi-la-v4-sur-stabilise)
14. [Corrections proposées](#14-corrections-proposées)

---

## 1. Propagation du signal

**Fichier** : `propagation.rs`

### 1.1 Contribution source

Pour chaque nœud source $i$ actif :

$$
S_i = a_i \cdot \text{sign}_i \cdot (1 + g_{\text{mod},i}) \cdot G
$$

où :
- $a_i$ = activation du nœud $i$
- $\text{sign}_i = +1$ si excitateur, $-g_{\text{inh}}$ si inhibiteur
- $g_{\text{mod},i}$ = modulation de gain par la zone PID (0 si pas de zone)
- $G$ = gain global de propagation

**Config** : $G = 0.6$, $g_{\text{inh}} = 0.8$

### 1.2 Noyau spatial

Pour chaque arête $i \to j$ de distance $d_{ij}$ :

**Exponentiel** :
$$K(d_{ij}) = \exp(-\lambda \cdot d_{ij})$$

**Gaussien** :
$$K(d_{ij}) = \exp\!\left(-\frac{1}{2}(\lambda \cdot d_{ij})^2\right)$$

où $\lambda$ = `spatial_decay`.

**Config** : $\lambda = 0.15$ (exponentiel)

### 1.3 Influence reçue

$$
I_j = \sum_{i \in \text{in}(j)} S_i \cdot w_{ij} \cdot K(d_{ij})
$$

où $w_{ij}$ = conductance de l'arête $i \to j$.

### 1.4 Application

$$
a_j(t+1) = \text{clamp}\!\left(a_j(t) + I_j,\ 0,\ 10\right)
$$

### 1.5 Analyse numérique

Avec KNN $k=10$, $G=0.6$, $w_{ij} \approx 1.0$, $K \approx e^{-0.15 \cdot d}$ :
- Un nœud excitateur actif ($a=1.0$) envoie $S = 1.0 \times 1.0 \times 1.0 \times 0.6 = 0.6$
- Chaque voisin reçoit $\approx 0.6 \times 1.0 \times e^{-0.15d}$
- Pour $d \approx 5$ : $I \approx 0.6 \times 0.47 = 0.28$ par arête
- 10 voisins tous actifs : $I_j \approx 2.8$ → **très fort**, dépasse le seuil (0.2) en un tick

---

## 2. Dynamique des nœuds

**Fichier** : `node.rs`

### 2.1 Seuil effectif

$$
\theta_{\text{eff},i} = \frac{\theta_i + F_i + H_i + \theta_{\text{mod},i}}{\max(\epsilon_i,\ 0.01)}
$$

où :
- $\theta_i$ = seuil de base (config: 0.2)
- $F_i$ = fatigue
- $H_i$ = inhibition locale
- $\theta_{\text{mod},i}$ = modulation PID indirect
- $\epsilon_i$ = excitabilité

### 2.2 Condition d'activité

$$
\text{actif}_i \iff a_i > \theta_{\text{eff},i}
$$

### 2.3 Dissipation de l'activation

$$
a_i(t+1) = \max\!\left(a_i(t) \cdot (1 - \alpha_{\text{eff},i}),\; a_{\min}\right)
$$

avec jitter déterministe :
$$
\alpha_{\text{eff},i} = \text{clamp}\!\left(\alpha \cdot (1 + J_i),\ 0,\ 1\right)
$$
$$
J_i = \left(\frac{h(i, t) \gg 33}{2^{31}} - 1\right) \cdot j_{\text{max}}
$$

où $h(i,t)$ est un hash LCG rapide. $\alpha = 0.25$, $j_{\text{max}} = 0.15$, $a_{\min} = 0.01$.

**Plages** : $\alpha_{\text{eff}} \in [0.2125, 0.2875]$

### 2.4 Fatigue

$$
F_i(t+1) = \begin{cases}
\text{clamp}\!\left((F_i + g_F \cdot a_i)(1 - r_F),\ 0,\ 10\right) & \text{si actif} \\
\text{clamp}\!\left(F_i \cdot (1 - r_F),\ 0,\ 10\right) & \text{sinon}
\end{cases}
$$

**Config** : $g_F = 0.20$, $r_F = 0.05$

### 2.5 Inhibition locale

$$
H_i(t+1) = \begin{cases}
\text{clamp}\!\left((H_i + g_H)(1 - d_H),\ 0,\ 10\right) & \text{si actif} \\
\text{clamp}\!\left(H_i \cdot (1 - d_H),\ 0,\ 10\right) & \text{sinon}
\end{cases}
$$

**Config** : $g_H = 0.12$, $d_H = 0.03$

### 2.6 Trace mémoire

$$
M_i(t+1) = \begin{cases}
\text{clamp}\!\left((M_i + g_M \cdot a_i)(1 - d_M),\ 0,\ 100\right) & \text{si actif} \\
\text{clamp}\!\left(M_i \cdot (1 - d_M),\ 0,\ 100\right) & \text{sinon}
\end{cases}
$$

**Config** : $g_M = 0.1$, $d_M = 0.005$

### 2.7 Excitabilité

$$
\epsilon_i = 1 + 0.1 \cdot \min(M_i, 5)
$$

**Plages** : $\epsilon_i \in [1.0, 1.5]$

### 2.8 Injection de stimulus

$$
a_i(t) \mathrel{+}= I_{\text{stimulus}}
$$

(Pas de clamp ici — le clamp se fait dans la propagation.)

---

## 3. Dynamique des arêtes — Plasticité hebbienne

**Fichier** : `edge.rs`, appelé depuis `stdp.rs`

### 3.1 Trace de co-activation

**Enregistrement** (si $a_{\text{src}} > 0.1$ ET $a_{\text{tgt}} > 0.1$) :
$$
C_{ij}(t) = \min\!\left(C_{ij}(t) + \min(a_{\text{src}}, 1) \cdot \min(a_{\text{tgt}}, 1),\ 10\right)
$$

**Décroissance** :
$$
C_{ij}(t+1) = C_{ij}(t) \cdot (1 - d_C)
$$

**Config** : $d_C = 0.05$

### 3.2 Mise à jour de conductance (Hebbian)

$$
w_{ij}(t+1) = \begin{cases}
w_{ij}(t) + \eta_+ \cdot p \cdot (C_{ij} - C_{\text{seuil}}) & \text{si } C_{ij} > C_{\text{seuil}} \\
w_{ij}(t) - \eta_- \cdot p & \text{si } C_{ij} \leq C_{\text{seuil}} \text{ et non consolidé} \\
w_{ij}(t) & \text{si consolidé et } C_{ij} \leq C_{\text{seuil}}
\end{cases}
$$

puis :
$$
w_{ij} = \text{clamp}(w_{ij},\ w_{\min},\ w_{\max})
$$

**Config** : $\eta_+ = 0.01$, $\eta_- = 0.002$, $C_{\text{seuil}} = 0.2$, $p = 1.0$, $w_{\min} = 0.1$, $w_{\max} = 5.0$

### 3.3 Analyse numérique — Hebbian

Si deux nœuds co-activent avec $a = 1.0$ :
- $C_{ij} += 1.0$ (puis decay ×0.95)
- $\Delta w_{\text{hebb}} = 0.01 \times 1.0 \times (1.0 - 0.2) = +0.008$ par tick
- Sinon : $\Delta w_{\text{hebb}} = -0.002$ par tick

**→ Taux de renforcement 4× supérieur à l'affaiblissement**, ce qui pousse naturellement $w$ vers $w_{\max} = 5.0$.

### 3.4 Décroissance de conductance (non utilisée en V3/V4)

$$
w_{ij}(t+1) = w_{ij}(t) + d_w \cdot (w_0 - w_{ij}(t))
$$

(Présent dans le code mais non appelé dans le tick — `edge.decay_conductance` n'est jamais invoqué.)

---

## 4. STDP

**Fichier** : `stdp.rs`

### 4.1 Fenêtre temporelle

Si le nœud pré-synaptique a fired au tick $t_{\text{pre}}$ et le post-synaptique au tick $t_{\text{post}}$ :

$$
\Delta t = t_{\text{post}} - t_{\text{pre}}
$$

$$
\Delta w_{\text{STDP}} = \begin{cases}
A^+ \cdot \exp\!\left(-\frac{\Delta t}{\tau^+}\right) & \text{si } \Delta t > 0 \text{ (pré avant post → LTP)} \\
-A^- \cdot \exp\!\left(\frac{\Delta t}{\tau^-}\right) & \text{si } \Delta t < 0 \text{ (post avant pré → LTD)}
\end{cases}
$$

$$
w_{ij} = \text{clamp}\!\left(w_{ij} + \Delta w_{\text{STDP}},\ w_{\min},\ w_{\max}\right)
$$

**Config** : $A^+ = A^- = 0.005$, $\tau^+ = \tau^- = 20$ ticks

### 4.2 Analyse numérique — STDP

- $\Delta t = 1$ tick : $\Delta w = 0.005 \times e^{-1/20} = 0.005 \times 0.951 = 0.00476$
- $\Delta t = 5$ ticks : $\Delta w = 0.005 \times e^{-5/20} = 0.005 \times 0.779 = 0.00389$
- $\Delta t = 20$ ticks : $\Delta w = 0.005 \times e^{-1} = 0.00184$

**→ Ordre de grandeur comparable au Hebbian (0.008), mais appliqué chaque tick de manière cumulative.**

### 4.3 Problème : `last_activation_tick` ne s'efface jamais

Le code met à jour `last_activation_tick = Some(tick)` quand le nœud est actif, mais **ne le remet jamais à `None`**. Donc un nœud qui a été actif une seule fois au tick 30 garde `last_activation_tick = Some(30)` indéfiniment. Conséquence : la STDP calcule des $\Delta t$ de plus en plus grands pour des paires de nœuds qui n'ont plus de relation temporelle réelle. Les $\Delta w$ diminuent exponentiellement, mais elles ne sont **jamais exactement zéro**.

---

## 5. Traces d'éligibilité (V4)

**Fichier** : `stdp.rs` (section V4)

### 5.1 Accumulation

$$
e_{ij}(t+1) = \text{clamp}\!\left(\gamma_e \cdot e_{ij}(t) + \Delta w_{\text{STDP}},\ -e_{\max},\ e_{\max}\right)
$$

**Config** : $\gamma_e = 0.95$, $e_{\max} = 5.0$

### 5.2 Analyse numérique — Éligibilité

Décroissance : $e_{ij}$ perd 5% par tick → demi-vie $\approx$ 14 ticks.

Accumulation en régime permanent si $\Delta w_{\text{STDP}} = \delta$ constant :
$$
e_\infty = \frac{\delta}{1 - \gamma_e} = \frac{\delta}{0.05} = 20\delta
$$

Pour $\delta = 0.005$ : $e_\infty = 0.1$

**→ Trace d'éligibilité moyenne observée : 0.04** (données métriques), cohérent.

---

## 6. Dopamine (V4)

**Fichier** : `dopamine.rs`

### 6.1 Mise à jour phasique

$$
d_{\text{pha}}(t+1) = (1 - \alpha_d) \cdot d_{\text{pha}}(t) + g_r \cdot r(t)
$$
$$
d_{\text{pha}} = \text{clamp}(d_{\text{pha}},\ -d_{\max},\ d_{\max})
$$

### 6.2 Niveau total

$$
d(t) = d_{\text{ton}} + d_{\text{pha}}(t)
$$

**Config** : $d_{\text{ton}} = 0.1$, $\alpha_d = 0.15$, $g_r = 1.0$, $d_{\max} = 2.0$

### 6.3 Analyse numérique — Dopamine

- Reward positif $r = +1.0$ : $d_{\text{pha}} += 1.0$ → $d = 0.1 + 1.0 = 1.1$
- Reward négatif $r = -0.5$ : $d_{\text{pha}} += -0.5$ → $d$ peut devenir négatif
- Demi-vie du phasique : $\ln 2 / 0.15 \approx 4.6$ ticks
- Après 10 ticks sans reward : $d_{\text{pha}} \times (0.85)^{10} = 0.197 \times d_{\text{pha}}$

**→ Signal dopaminergique : disparaît en ~10 ticks après un reward.**

### 6.4 PROBLÈME CRITIQUE : dopamine update timing

Dans `simulation.rs`, la dopamine est mise à jour **UNIQUEMENT** au moment du readout (tick précis du readout). Mais la modulation dopamine → plasticité se fait à **chaque tick**. Donc :

- Ticks 1→34 (readout tick du trial 1) : $d = d_{\text{ton}} = 0.1$ (phasique = 0)
- Tick 35 (readout) : reward → $d_{\text{pha}} = ±1.0$, $d = 1.1$ ou $-0.4$
- Ticks 36→55 : $d_{\text{pha}}$ décroît de 15%/tick... **MAIS** il n'y a pas de `dopamine.update(0.0)` entre les readouts !

Le code actuel ne fait `dopamine.update(0.0)` que dans le `else` de `output_reader`, et seulement si `reward_system.current == 0.0`. **La composante phasique ne décroît pas entre les trials.**

Vérifié : dans les données, `dopa` reste élevé entre les trials (ex : 2.1 au tick 400, 1.542 au tick 700) au lieu de retomber rapidement.

---

## 7. Modulation reward → conductance (V4)

**Fichier** : `stdp.rs` (section V4, dans la boucle parallèle)

### 7.1 Formule

$$
\Delta w_{\text{reward}} = \eta_{\text{rew}} \cdot d(t) \cdot e_{ij}(t)
$$
$$
w_{ij} = \text{clamp}\!\left(w_{ij} + \Delta w_{\text{reward}},\ w_{\min},\ w_{\max}\right)
$$

**Config** : $\eta_{\text{rew}} = 0.01$

### 7.2 Analyse numérique — PROBLÈME MAJEUR

Calculons $\Delta w_{\text{reward}}$ dans un cas favorable (juste après reward positif) :

$$
\Delta w_{\text{reward}} = 0.01 \times 1.1 \times 0.04 = 0.00044
$$

Comparons avec la plasticité hebbienne par tick :
$$
\Delta w_{\text{hebb}} = 0.01 \times 0.8 = 0.008
$$

Et la STDP par tick :
$$
\Delta w_{\text{STDP}} \approx 0.005
$$

**Rapport de forces** :

| Source | $\Delta w$ / tick | Occurrence |
|--------|-------------------|------------|
| Hebbian | +0.008 | Chaque tick, toute arête co-active |
| STDP | ±0.005 | Chaque tick, toute arête avec timing |
| Reward | ±0.00044 | Chaque tick, mais $e_{ij}$ décroît vite |
| Budget | ÷ facteur | Chaque tick, compresse vers $w_{\max}/k$ |

**→ Le signal de reward est 18× plus faible que le Hebbian et 11× plus faible que la STDP.**

Le reward est complètement noyé dans le bruit de la plasticité non-supervisée. C'est la **cause principale** de la non-discrimination entre classes.

---

## 8. Budget synaptique

**Fichier** : `stdp.rs`

### 8.1 Calcul

Pour chaque nœud source $i$, calculer le total sortant :
$$
W_i = \sum_{j \in \text{out}(i)} w_{ij}
$$

Si $W_i > B$ et l'arête n'est pas consolidée :
$$
w_{ij} \leftarrow \max\!\left(w_{ij} \cdot \frac{B}{W_i},\ w_{\min}\right)
$$

**Config** : $B = 30.0$

### 8.2 Analyse numérique — PROBLÈME MAJEUR

Avec KNN $k = 10$ voisins, le budget par arête est au maximum :
$$
w_{\text{budget}} = \frac{B}{k} = \frac{30}{10} = 3.0
$$

**Mais** les arêtes sont bidirectionnelles (KNN symétrique en 3D), donc le degré effectif est $\sim 10$ sortant. Observation : `mean_conductance` converge rapidement vers **2.75** (données métriques) ce qui correspond à $27.5/10$ = budget quasi-saturé.

Le budget force toutes les conductances vers la même valeur ($\approx 2.75$) **quelle que soit l'activité**. L'information différentielle entre arêtes renforcées (par reward) et arêtes neutres est **systématiquement effacée** par la normalisation.

### 8.3 Interaction avec consolidation — AMPLIFICATEUR DE PROBLÈME

Données métriques :
- Tick 90 : 5 arêtes consolidées, mean_cond = 2.68
- Tick 100 : **29 950** arêtes consolidées, mean_cond = 2.68
- Tick 110 : **76 871** arêtes consolidées
- Tick 130 : **128 923** arêtes consolidées
- Tick 200+ : **~171 000** arêtes consolidées (30% du total 577 488)

**Le problème** : quand une arête est consolidée ($w \geq 2.5$ pendant 50 ticks), le budget ne peut plus la comprimer. Mais le budget total reste 30, et les arêtes non-consolidées doivent se partager le reste. Comme pratiquement toutes les arêtes actives atteignent $w = 2.5$ (poussées par Hebbian+STDP pendant que le budget maintient $w \approx 2.75$), elles se consolident massivement.

**→ Après ~200 ticks, 30% des arêtes sont consolidées à $w \approx 2.75$, figées pour toujours.** Le réseau perd toute capacité de réorganisation.

---

## 9. Consolidation

**Fichier** : `edge.rs`

### 9.1 Compteur

$$
\text{counter}_{ij}(t+1) = \begin{cases}
\text{counter}_{ij}(t) + 1 & \text{si } w_{ij} \geq \theta_{\text{consol}} \\
0 & \text{sinon}
\end{cases}
$$

$$
\text{consolidated}_{ij} = \begin{cases}
\text{true} & \text{si counter}_{ij} \geq T_{\text{consol}} \\
\text{consolidated}_{ij} & \text{sinon}
\end{cases}
$$

**Config** : $\theta_{\text{consol}} = 2.5$, $T_{\text{consol}} = 50$ ticks

### 9.2 Gate V4

En V4 avec dopamine + éligibilité activés :
$$
\text{may consolidate} \iff d(t) > d_{\text{consol}} \wedge e_{ij} > 0
$$

**Config** : $d_{\text{consol}} = 0.3$

### 9.3 Analyse numérique — PROBLÈME

Le seuil de consolidation $\theta_{\text{consol}} = 2.5$ est inférieur à la conductance moyenne du budget ($\approx 2.75$).

**→ Toute arête qui atteint l'état d'équilibre du budget ($w \approx 2.75$) dépasse automatiquement le seuil de consolidation ($2.5$).** Il suffit de 50 ticks au-dessus pour consolider.

Le gate V4 ($d > 0.3$ ET $e > 0$) ne protège pas suffisamment :
- $d_{\text{ton}} = 0.1 < 0.3$ ✓ (gate fermé au repos)
- Mais dès qu'un reward arrive : $d = 1.1$, et $e_{ij}$ est positif pour presque toute arête qui a eu un delta STDP. Le gate s'ouvre massivement.

---

## 10. Contrôle PID par zones

**Fichier** : `zone.rs`

### 10.1 Mesure

$$
\bar{a}_z = \frac{1}{|Z_z|} \sum_{i \in Z_z} a_i
$$

### 10.2 PID

$$
e_z(t) = a_{\text{target}} - \bar{a}_z(t)
$$
$$
I_z(t) = \text{clamp}\!\left(I_z(t-1) + e_z(t),\ -I_{\max},\ I_{\max}\right)
$$
$$
D_z(t) = e_z(t) - e_z(t-1)
$$
$$
u_z(t) = \text{clamp}\!\left(K_p \cdot e_z + K_i \cdot I_z + K_d \cdot D_z,\ -u_{\max},\ u_{\max}\right)
$$

**Config** : $a_{\text{target}} = 0.3$, $K_p = 0.5$, $K_i = 0.05$, $K_d = 0.1$, $I_{\max} = 5$, $u_{\max} = 2$

### 10.3 Mode indirect (V3/V4)

$$
\theta_{\text{mod},z} = -k_\theta \cdot u_z
$$
$$
g_{\text{mod},z} = +k_g \cdot u_z
$$

Appliqué à chaque nœud $i \in Z_z$ :
- $\theta_{\text{mod},i} = \theta_{\text{mod},z}$
- $g_{\text{mod},i} = g_{\text{mod},z}$ (utilisé dans propagation §1.1)

**Config** : $k_\theta = 0.3$, $k_g = 0.2$

### 10.4 Analyse numérique

Avec $\bar{a} = 3.6$ (énergie 180k / 50k nœuds) :
$$
e = 0.3 - 3.6 = -3.3
$$
$$
u = 0.5 \times (-3.3) + 0.05 \times I + 0.1 \times D = -1.65 + \ldots
$$
$u$ est clampé à $-2.0$.
$$
\theta_{\text{mod}} = -0.3 \times (-2.0) = +0.6
$$
$$
g_{\text{mod}} = 0.2 \times (-2.0) = -0.4
$$

Seuil effectif avec PID : $(0.2 + F + H + 0.6) / \epsilon \approx (0.2 + F + H + 0.6) / 1.3$

Le PID pousse le seuil vers le haut et le gain vers le bas, ce qui réduit l'activité. **Mais** avec une énergie de 180k, le PID est en saturation permanente ($u = -2.0$). Il ne peut pas plus freiner.

**→ Activité moyenne 3.6 vs cible 0.3 : le PID est dépassé d'un facteur 12x.** La saturation à $u_{\max} = 2.0$ empêche une régulation suffisante.

---

## 11. Pacemakers

**Fichier** : `pacemaker.rs`

$$
a_i(t) \mathrel{+}= A \cdot \sin(2\pi f t + \phi) + O
$$

**Config V4** : aucun pacemaker configuré.

---

## 12. Entrée / Sortie (V4)

**Fichier** : `input.rs`, `output.rs`

### 12.1 Injection d'entrée

Pour chaque nœud $i$ du groupe de la classe $c$ pendant $T_{\text{présentation}}$ ticks :
$$
a_i(t) \mathrel{+}= I_{\text{input}}
$$

**Config** : $I_{\text{input}} = 1.5$, 50 nœuds/classe, 5 ticks de présentation

### 12.2 Readout de sortie

Pour chaque classe de sortie $c$ :
$$
\text{score}_c = \sum_{i \in \text{out}(c)} a_i
$$
$$
\text{decision} = \arg\max_c\ \text{score}_c
$$

### 12.3 Analyse numérique — Entrée

50 nœuds reçoivent $+1.5$ par tick pendant 5 ticks.
Avec propagation gain = 0.6 et KNN $k=10$ :
- Tick 1 : 50 nœuds à $a = 1.5$
- Tick 2 : 50 nœuds d'entrée + ~500 voisins activés par propagation
- Tick 5 : cascade à travers le réseau

En KNN 3D avec 50k nœuds et $k=10$, la distance moyenne inter-voisins est faible ($\sim 5-10$). Le signal atteint tous les 50k nœuds en ~15-20 ticks.

**→ Pattern d'entrée local (50 nœuds) → activation globale (50k nœuds).** Pas de localité préservée.

---

## 13. Diagnostic : Pourquoi la V4 sur-stabilise

Les données métriques (section §7.2) montrent que le réseau atteint un état figé après ~150 ticks :

| Symptôme | Valeur | Attendu |
|----------|--------|---------|
| Conductance moyenne | 2.75 (constant) | Variable |
| Conductance max | 5.0 (saturé) | < 5.0 |
| Arêtes consolidées | 171k / 577k (30%) dès tick 200 | Quelques % |
| Activité | 180k énergie (stable) | Variable avec patterns |
| Accuracy | 45% (hasard) | > 60% progressive |
| Éligibilité moyenne | 0.04 (constante) | Variable |

### 13.1 Causes racines identifiées

#### CAUSE 1 : Le reward est 18× plus faible que la plasticité non-supervisée

$$
\frac{|\Delta w_{\text{reward}}|}{|\Delta w_{\text{hebb}}|} = \frac{0.01 \times 1.1 \times 0.04}{0.008} = \frac{0.00044}{0.008} = 0.055
$$

La plasticité hebbienne et la STDP s'appliquent à chaque tick sur toutes les arêtes co-actives, sans discrimination de pattern. Le signal de reward modulé par éligibilité ne représente que **5.5%** de la plasticité totale. Il est noyé.

#### CAUSE 2 : Le budget efface la différenciation

Le budget synaptique ($B = 30$) normalise les conductances sortantes de chaque nœud. Avec 10 voisins, toutes les conductances convergent vers $30/10 = 3.0$. Même si le reward renforce spécifiquement certaines arêtes, le budget les recomprime au tick suivant.

$$
\text{Après reward} : w_{ij} = 2.75 + 0.00044 = 2.75044
$$
$$
\text{Après budget} : w_{ij} = 2.75044 \times \frac{30}{27.50044} \approx 2.75
$$

**Le budget annule exactement l'effet du reward.**

#### CAUSE 3 : Consolidation massive et prématurée

Le seuil de consolidation ($\theta_{\text{consol}} = 2.5$) est inférieur à la conductance d'équilibre ($\approx 2.75$). **Toute arête active pendant 50 ticks se consolide**. Une fois consolidée, elle est :
- Exclue de l'affaiblissement hebbien
- Exclue de la normalisation budget

En 200 ticks, 30% du réseau est figé. Le gate V4 (dopamine > 0.3) ne protège pas car $d_{\text{ton}} + d_{\text{pha}}$ dépasse 0.3 fréquemment.

#### CAUSE 4 : La dopamine ne décroît pas entre les trials

Le code ne fait **pas** décroître la composante phasique entre deux readouts. La condition dans `simulation.rs` :

```rust
if config.dopamine.enabled && self.reward_system.current == 0.0 {
    self.dopamine_system.update(0.0, &config.dopamine);
}
```

est dans un `else` (quand il n'y a PAS d'output reader), donc elle n'est **jamais exécutée** en mode V4 avec output. La dopamine phasique reste élevée indéfiniment après le premier reward.

#### CAUSE 5 : L'entrée inonde le réseau entier

Avec 50 nœuds à $I = 1.5$ et propagation $G = 0.6$ sur un graphe KNN connecté, le signal se propage dans tout le réseau en ~15 ticks. Les patterns de classe 0 et classe 1 activent le **même ensemble global de nœuds**, empêchant toute discrimination topologique.

#### CAUSE 6 : Le PID est en saturation permanente

L'activité moyenne ($\bar{a} \approx 3.6$) est 12× supérieure à la cible PID ($a_{\text{target}} = 0.3$). Le PID est clampé à $u_{\max} = -2.0$ et ne peut pas corriger davantage. Les modulations de seuil ($+0.6$) et gain ($-0.4$) sont insuffisantes.

---

## 14. Corrections proposées

### 14.1 Fix immédiat : décroissance dopamine à chaque tick

```
// À chaque tick, pas seulement au readout :
self.dopamine_system.update(0.0, &config.dopamine);
// Puis, au readout, le reward s'ajoute :
self.dopamine_system.update(reward_val, &config.dopamine);
```

### 14.2 Fix : rapport reward / Hebbian

Soit réduire la plasticité non-supervisée en mode V4, soit augmenter $\eta_{\text{rew}}$ :

$$
\eta_{\text{rew}} \geq \frac{\Delta w_{\text{hebb}}}{d_{\text{max}} \cdot e_{\text{typical}}} = \frac{0.008}{1.1 \times 0.04} = 0.18
$$

→ $\eta_{\text{rew}} = 0.2$ (au lieu de 0.01) pour une parité reward/Hebbian.

OU : modérer le Hebbian par la dopamine :
$$
\Delta w_{\text{hebb,V4}} = \Delta w_{\text{hebb}} \cdot \min(d(t) / d_{\text{ton}},\ 1)
$$

### 14.3 Fix : seuil de consolidation

$$
\theta_{\text{consol}} > \frac{B}{k} = \frac{30}{10} = 3.0
$$

→ $\theta_{\text{consol}} = 3.5$ ou $4.0$ pour que seules les arêtes spécifiquement renforcées (au-dessus du budget moyen) se consolident.

### 14.4 Fix : localité de l'entrée

Réduire le gain de propagation ou augmenter l'atténuation spatiale pour préserver la localité des patterns :
- $G = 0.3$ au lieu de 0.6
- $\lambda = 0.3$ au lieu de 0.15

Ou bien utiliser des nœuds d'entrée spatialement séparés pour chaque classe (au lieu de contigus).

### 14.5 Fix : PID cible vs activation réelle

Ajuster $a_{\text{target}}$ à la réalité du réseau V4 avec injection :
- $a_{\text{target}} = 2.0$ au lieu de 0.3
- Ou augmenter $u_{\max}$ pour que le PID puisse effectivement réguler.

---

## 15. Corrections implémentées (post-diagnostic)

### Fix 1 — Décroissance dopamine chaque tick (BUG critique)
**Fichier modifié** : `simulation.rs`

Le bug : `dopamine_system.update(0.0, &config)` était dans un `else` jamais atteint
quand `output_reader` est défini. La dopamine phasique ne décroissait donc **jamais**.

**Correction** : appel `update(0.0)` à chaque tick (phase 5b), AVANT la plasticité.
Le burst de reward est injecté séparément au readout (phase 8).

$$d_{\text{pha}}(t+1) = (1-\alpha_d) \cdot d_{\text{pha}}(t) + g_r \cdot r(t)$$

Avec $\alpha_d = 0.15$, le phasique décroît de 85% par tick entre les trials.
À chaque readout, $r(t)$ est injecté (+1.0 ou -0.5).

### Fix 2 — Gain de modulation reward : 0.01 → 0.5
**Fichier modifié** : `config_v4_reward.toml` `[dopamine]`

Avant : $\Delta w_{\text{reward}} = 0.01 \cdot d(t) \cdot e_{ij} \approx 0.00044$
Après : $\Delta w_{\text{reward}} = 0.5 \cdot d(t) \cdot e_{ij} \approx 0.022$

Rapport reward/Hebbien passe de 1:18 à ~2:1 (reward dominant).

### Fix 3 — Seuil de consolidation : 2.5 → 4.0
**Fichier modifié** : `config_v4_reward.toml` `[consolidation]`

L'équilibre budget est $B/k = 30/10 = 3.0$. Avec seuil à 2.5, la grande majorité
des arêtes consolidaient prématurément (171k/577k à t=200).

Avec seuil 4.0 > 3.0, seules les arêtes sélectivement renforcées par le reward
atteignent la consolidation. Résultat : 512 arêtes consolidées à t=2000.

### Fix 4 — Gain de propagation : 0.6 → 0.3
**Fichier modifié** : `config_v4_reward.toml` `[propagation]`

Avec $G = 0.6$, le signal d'entrée (50 nœuds) activait ~7500/50000 nœuds,
noyant la spécificité spatiale. Avec $G = 0.3$, l'activité reste plus localisée
autour des groupes d'entrée, améliorant le rapport signal/bruit.

### Fix 5 — Cible PID : 0.3 → 2.0
**Fichier modifié** : `config_v4_reward.toml` `[zones]`

L'activité moyenne réelle étant ~3.6, le PID saturait à $u = -2.0$ en permanence.
Avec cible 2.0, le PID opère dans sa zone linéaire et régule effectivement.

### Fix 6 — Gating Hebbien par dopamine
**Fichier modifié** : `stdp.rs`

La plasticité Hebbienne est maintenant modulée par le niveau dopaminergique :

$$\text{gate} = \text{clamp}\left(\frac{d_{\text{total}}}{d_{\text{tonic}}},\ 0.1,\ 2.0\right)$$

$$\eta_+^{\text{eff}} = \eta_+ \cdot \text{gate}, \quad \eta_-^{\text{eff}} = \eta_- \cdot \text{gate}$$

Effet :
- Entre trials ($d \approx d_{\text{tonic}}$) → gate ≈ 1.0 (normal)
- Après succès ($d \gg d_{\text{tonic}}$) → gate = 2.0 (renforcé)
- Après échec ($d < 0$) → gate = 0.1 (quasi-supprimé)

### Fix 7 — Budget synaptique à intervalle lent
**Fichier modifié** : `config.rs` (struct `SynapticBudgetConfig`) + `stdp.rs`

Nouveau paramètre `interval` (défaut 50 ticks). Le budget ne renormalise
que tous les 50 ticks au lieu de chaque tick, laissant le temps à la modulation
reward de s'accumuler avant d'être redistribuée.

### Fix 8 — Modulation reward APRÈS budget
**Fichier modifié** : `stdp.rs`

L'ordre des opérations dans le passage parallèle est maintenant :
1. Hebbien (gaté par dopamine)
2. STDP + accumulation éligibilité
3. Consolidation (gatée par dopamine + éligibilité)
4. Budget synaptique (intervalle lent)
5. **Modulation dopamine** (APRÈS budget → le signal reward survit)

$$w_{ij}^{\text{final}} = w_{ij}^{\text{post-budget}} + \eta_{\text{rew}} \cdot d(t) \cdot e_{ij}$$

### Fix 9 — Taux Hebbien réduit
**Fichier modifié** : `config_v4_reward.toml` `[plasticity]`

- `reinforcement_rate` : 0.01 → 0.002
- `weakening_rate` : 0.002 → 0.0005

Ratio reward/Hebbien final ≈ 10:1. Le système est désormais piloté
principalement par le signal de récompense.

---

## 16. Résultats de validation post-corrections

| Mesure                     | Avant fixes | Après fixes |
|----------------------------|:-----------:|:-----------:|
| Accuracy finale            |   ~45%      |   **54%**   |
| Peak accuracy (début)      |   ~50%      |   **100%**  |
| Dopamine dynamique         |   Non (≈1.1)| **Oui** [-0.4, +1.1] |
| Arêtes consolidées (t=2000)|  171 000    |   **512**   |
| Énergie (variation)        | ±5k (gelé)  | **±30k** (dynamique) |
| Temps/tick                  |  ~13ms      |  ~19ms      |

**Observation clé** : la learning curve montre un pic initial fort (100% à t=100),
puis une décroissance vers ~54%. Cela indique que :
1. Le mécanisme de reward fonctionne correctement
2. L'oubli catastrophique (Hebbian + budget) érode les patterns appris
3. Le plafond à 54% est lié à la topologie du réseau (graphe KNN aléatoire 3D)
   avec un ratio entrée/sortie de 0.2% (100 nœuds sur 50000)

**Prochaines pistes (V4.1)** :
- Routing spatial : placer les nœuds input/output dans des régions proches
  pour créer des chemins courts et fiables
- Soft budget : remplacer le hard-cap par une pénalité douce ($L_1$ regularization)
- Multi-trial consolidation : ne consolider qu'après N succès consécutifs

---

## 17. Diagnostic fondamental : pourquoi les corrections 1-9 ne suffisent pas

Les 9 corrections (§15) traitaient les symptômes (rapport reward/Hebbian, budget,
consolidation, etc.) mais pas la **cause architecturale** :

> **Le système a 3 forces de plasticité concurrentes qui modifient toutes la conductance :**
> 1. Hebbienne (co-activité) → Δw = +η₊ × (C_ij - seuil) **chaque tick, non supervisé**
> 2. STDP (timing) → Δw = ±A × exp(-Δt/τ) **chaque tick, non supervisé**
> 3. Reward (3-facteur) → Δw = η × d × e **seulement lors d'un burst dopamine**
>
> Les forces 1 et 2 sont **non supervisées** : elles renforcent ce qui co-active
> ou ce qui est temporellement causal, **sans distinguer les chemins utiles des parasites**.
> La force 3 (reward) est la seule force orientée vers l'objectif.
>
> **Quand 2 forces non-supervisées dominent 1 force supervisée, l'apprentissage guidé
> est impossible.** Ajuster les gains ne résout pas le problème structurel.

### 17.1 Le modèle biologique correct : la règle des trois facteurs

Référence : Frémaux & Gerstner (2016), *Neuromodulated STDP, and Theory of Three-Factor
Learning Rules*. Neuron.

Dans le cerveau, la plasticité synaptique long-terme (LTP/LTD) nécessite **trois signaux
simultanés** :

1. **Activité pré-synaptique** (le neurone source a tiré)
2. **Activité post-synaptique** (le neurone cible a tiré)
3. **Signal neuromodulateur** (dopamine, sérotonine, acétylcholine)

**Sans le troisième facteur (dopamine), la STDP ne produit que des changements temporaires
qui s'effacent.** C'est le mécanisme biologique qui résout le **problème du crédit temporel
distal** : l'activité locale crée une *trace d'éligibilité*, et c'est le signal de
récompense (arrivant plus tard) qui convertit cette trace en modification permanente.

### 17.2 Conséquence pour Stretch V4

La plasticité **Hebbienne** et la **STDP directe** (modification immédiate de conductance)
doivent être **supprimées** en V4. Elles sont remplacées par :

- **STDP → éligibilité** : la STDP calcule une direction d'apprentissage ψ mais ne touche
  **pas** à la conductance. Elle alimente uniquement la trace d'éligibilité.
- **Trois facteurs → conductance** : seul le produit `η × δ_d × e_ij` modifie la
  conductance, où `δ_d = d(t) - d_tonic` est le signal de surprise dopaminergique.
- **Décroissance homéostatique** : lente dérive de C vers une baseline, remplaçant le budget
  et l'affaiblissement Hebbien.

---

## 18. Modèle mathématique corrigé — Chemins dopaminergiques

### 18.1 Propagation (inchangée, paramètres corrigés)

$$
I_j = \sum_{i \in \text{actifs\_entrants}(j)} a_i \cdot \text{sign}_i \cdot (1 + g_{\text{mod},i}) \cdot G \cdot C_{ij} \cdot K(d_{ij})
$$

$$
a_j(t+1) = \text{clamp}(a_j(t) + I_j, 0, 10)
$$

**Paramètre clé** : $G = 0.3$.

**Justification** : avec une activité sparse (~5% de nœuds actifs) et $C = 1.0$ (initial) :
- Influence d'un voisin actif : $I = 1.0 \times 0.3 \times 1.0 \times 0.47 = 0.14 < \theta = 0.2$ → ne cascade pas
- Avec un chemin renforcé $C = 3.0$ : $I = 1.0 \times 0.3 \times 3.0 \times 0.47 = 0.42 > \theta$ → propage !
- **La conductance discrimine les chemins.** Les connexions fortes propagent, les faibles non.

### 18.2 Direction STDP (ne modifie PAS la conductance)

Pour chaque arête $i \to j$, la STDP calcule un signal directionnel :

$$
\psi_{ij}(t) = \begin{cases}
A^+ \exp\!\left(-\frac{\Delta t}{\tau^+}\right) & \text{si } \Delta t > 0 \text{ (pré→post : LTP)} \\[6pt]
-A^- \exp\!\left(\frac{\Delta t}{\tau^-}\right) & \text{si } \Delta t < 0 \text{ (post→pré : LTD)}
\end{cases}
$$

avec $\Delta t = t_\text{post} - t_\text{pre}$.

**Crucial** : ψ alimente la **trace d'éligibilité**, pas la conductance.

### 18.3 Trace d'éligibilité

$$
e_{ij}(t+1) = \gamma_e \cdot e_{ij}(t) + \psi_{ij}(t)
$$

avec $\gamma_e = 0.95$ (demi-vie ≈ 14 ticks).

L'éligibilité mémorise quelles synapses ont été causalement impliquées dans l'activité
récente. Elle s'accumule si un motif temporel se répète (pré→post), et décroît sinon.

### 18.4 Règle des trois facteurs (SEULE modification de conductance)

$$
\Delta C_{ij}(t) = \eta \cdot \delta_d(t) \cdot e_{ij}(t)
$$

où :
- $\eta = 0.5$ (plasticity\_gain, paramètre existant)
- $\delta_d(t) = d(t) - d_{\text{tonic}}$ est le **signal de surprise dopaminergique**

**Propriétés** :
- Entre les trials ($d \approx d_{\text{tonic}}$) : $\delta_d \approx 0$ → **pas d'apprentissage**
- Après succès ($d \gg d_{\text{tonic}}$) : $\delta_d > 0$ → **renforcement** des synapses éligibles
- Après échec ($d < d_{\text{tonic}}$) : $\delta_d < 0$ → **affaiblissement** des synapses éligibles

### 18.5 Décroissance homéostatique

Remplace le budget synaptique et l'affaiblissement Hebbien :

$$
C_{ij}(t+1) = C_{ij}(t) + \Delta C_{ij}(t) + \lambda_h \cdot (C_0 - C_{ij}(t))
$$

avec :
- $\lambda_h = 0.0001$ (très lent, demi-vie ≈ 7000 ticks)
- $C_0 = 1.0$ (conductance de base)

**Rôle** :
- Tire lentement les connexions non-renforcées vers $C_0 = 1.0$
- Ne détruit pas les chemins appris (l'apprentissage par reward est ~100× plus rapide)
- Fournit un oubli naturel pour les chemins qui ne sont plus récompensés

### 18.6 Dopamine

$$
d_{\text{pha}}(t+1) = (1 - \alpha_d) \cdot d_{\text{pha}}(t) + g_r \cdot r(t)
$$
$$
d(t) = d_{\text{tonic}} + d_{\text{pha}}(t)
$$
$$
\delta_d(t) = d(t) - d_{\text{tonic}} = d_{\text{pha}}(t)
$$

**Le signal de surprise EST la composante phasique.** C'est exactement le modèle de
Schultz (1997) : la dopamine code l'erreur de prédiction de récompense.

- Récompense inattendue → burst ($\delta_d > 0$) → LTP sur les chemins récemment actifs
- Punition → dip ($\delta_d < 0$) → LTD sur les chemins récemment actifs
- Récompense attendue → neutre ($\delta_d = 0$) → pas de changement

### 18.7 Consolidation

Gatée par le signal dopaminergique ET l'éligibilité (inchangé) :

$$
\text{may\_consolidate} \iff d(t) > d_{\text{consol}} \wedge e_{ij} > 0
$$

Seules les arêtes dont $C \geq 4.0$ pendant 50 ticks consécutifs (EN conditions de reward)
se consolident. Cela garantit que seuls les chemins consistamment récompensés sont figés.

---

## 19. Analyse numérique du modèle trois-facteurs

### 19.1 Amplitude d'apprentissage par trial

Après un succès ($r = +1$) :
- $d_{\text{pha}}(0) = 1.0$ → $\delta_d(0) = 1.0$
- $\delta_d$ décroît : $1.0, 0.85, 0.72, 0.61, 0.52, \ldots$
- Somme sur 10 ticks : $\sum \delta_d \approx 5.0$
- Éligibilité typique d'un chemin actif : $e \approx 0.05$
- $\sum \Delta C = \eta \cdot e \cdot \sum \delta_d = 0.5 \times 0.05 \times 5.0 = 0.125$

→ **+0.125 de conductance par trial réussi** sur les arêtes du chemin actif.

Après un échec ($r = -0.5$) :
- $\delta_d(0) = -0.5$, somme sur 10 ticks ≈ $-2.5$
- $\sum \Delta C = 0.5 \times 0.05 \times (-2.5) = -0.0625$

→ **-0.063 de conductance par trial échoué**.

### 19.2 Évolution de la conductance sur les chemins

| Trials réussis | C estimée | Notes |
|:-:|:-:|:--|
| 0 | 1.0 | Baseline |
| 10 | 2.25 | Signal commence à se propager (I > θ) |
| 20 | 3.50 | Chemin bien établi |
| 40 | 5.00 | Saturation (clamp à $C_{\max}$) |

Après ~20 trials réussis, le chemin est solide. C'est cohérent avec l'échelle de
la simulation (93 trials possibles en 2000 ticks avec période 21).

### 19.3 Décroissance homéostatique vs apprentissage

- Homéostatique par tick : $\Delta C = 0.0001 \times (1.0 - C)$
  - À $C = 5.0$ : $-0.0004$ par tick → $-0.008$ par trial (21 ticks)
- Apprentissage par trial : $+0.125$ (succès)
- **Ratio : apprentissage 15× plus fort que la décroissance** → les chemins tiennent.

### 19.4 Sparsité et discrimination

Avec $G = 0.3$, $\theta = 0.2$, et activité sparse (~5%) :
- Connexion $C = 1.0$ (non apprise) avec 1 voisin actif : $I = 0.14$ → **sous le seuil**
- Connexion $C = 3.0$ (apprise) avec 1 voisin actif : $I = 0.42$ → **au-dessus du seuil**
- **Le réseau route le signal SÉLECTIVEMENT à travers les chemins renforcés.**

C'est exactement le mécanisme des voies dopaminergiques : le signal voyage uniquement
à travers les synapses qui ont été renforcées par le reward.

### 19.5 PID et sparsité

Cible d'activité : $a_{\text{target}} = 0.1$ (activation moyenne, correspond à ~5% de
nœuds actifs).

Le PID maintient une activité basale sparse. Quand un input est injecté, l'activité
augmente localement mais ne cascade pas globalement (car $G = 0.3$ et $C = 1.0$ sont
sous-critiques). Le signal ne se propage que sur les chemins appris ($C \gg 1$).

---

## 20. Résumé des changements V4 corrigé

### Supprimé
- Plasticité Hebbienne (co-activité → conductance)
- STDP directe (timing → conductance)
- Budget synaptique dur (car remplacé par homéostasie + clamp)

### Conservé
- Propagation signée E/I
- PID indirect par zones
- Fatigue, inhibition, trace mémoire
- Consolidation (mais gatée par dopamine + éligibilité)
- Dissipation
- Dopamine tonique/phasique

### Ajouté/Modifié
- **STDP → éligibilité uniquement** (ne touche plus la conductance)
- **Règle des trois facteurs** : ΔC = η × δ_d × e (seule modification de C)
- **Décroissance homéostatique** : C → C₀ lentement (remplace budget + Hebbian weakening)
- **G = 0.3** (signaux sous-critiques, discrimination par conductance)
- **target_activity = 0.1** (activité sparse)

### Paramètres finaux

| Paramètre | Valeur | Rôle |
|:--|:-:|:--|
| $G$ (gain propagation) | 0.3 | Sous-critique pour C=1, super-critique pour C≥3 |
| $\theta$ (seuil de base) | 0.2 | Seuil d'activation |
| $\alpha$ (decay activation) | 0.25 | Dissipation par tick |
| $\eta$ (plasticity\_gain) | 0.5 | Taux d'apprentissage trois-facteurs |
| $\gamma_e$ (eligibility decay) | 0.95 | Demi-vie éligibilité ~14 ticks |
| $\lambda_h$ (homeostatic decay) | 0.0001 | Demi-vie ~7000 ticks |
| $C_0$ (conductance baseline) | 1.0 | Cible homéostatique |
| $a_{\text{target}}$ | 0.1 | Activité sparse (~5%) |
| $A^+, A^-$ | 0.005 | STDP amplitude |
| $d_{\text{tonic}}$ | 0.1 | Dopamine basale |
| $\alpha_d$ | 0.15 | Decay phasique |

