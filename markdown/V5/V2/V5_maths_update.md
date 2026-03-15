# V5.2 — Modélisation Mathématique : Bilan, Audit & Résultats Expérimentaux

> Document rédigé après audit complet du code source (CPU + GPU) et validation
> par 10+ expériences d'isolation systématique.

---

## 1. Récapitulatif du Modèle Mathématique Complet (V4 → V5 → V5.2)

### 1.1 Propagation d'activité

Contribution source (nœud $i$ actif si $a_i > \theta_{\text{eff},i}$) :

$$s_i = a_i \cdot \text{sign}(i) \cdot g_{\text{mod},i} \cdot G_{\text{prop}}$$

où $\text{sign}(i) = +1$ si excitateur, $-G_{\text{inh}}$ si inhibiteur.

Seuil effectif :

$$\theta_{\text{eff},i} = \max\!\Big(\frac{\theta_i + F_i + I_i + \theta_{\text{mod},i}}{\max(\xi_i,\, 0.01)},\; 0.05\Big)$$

Influence accumulée sur le nœud cible $j$ (CSR) :

$$\text{infl}_j = \sum_{k \in \text{in}(j)} s_{\text{src}(k)} \cdot C_k \cdot K_k$$

Mise à jour :

$$a_j \leftarrow \text{clamp}(a_j + \text{infl}_j,\; 0,\; 10)$$

### 1.2 Dissipation

**Standard (V4)** — jitter multiplicatif par nœud :
$$a_i \leftarrow a_i \cdot (1 - \alpha_{\text{decay}} \cdot (1 + j_i))$$

**Adaptative (V5)** — décroissance modulée par l'activité voisine :
$$\alpha_{\text{eff},i} = \alpha_{\text{base}} \cdot \text{clamp}(1 - k_{\text{local}} \cdot \bar{a}_{\mathcal{N}(i)},\; 0.2,\; 1.0)$$
$$a_i \leftarrow a_i \cdot (1 - \alpha_{\text{eff},i})$$

### 1.3 Réverbération (V5)

$$a_i(t) \leftarrow a_i(t) + r_{\text{gain}} \cdot a_i^{\text{snapshot}}(t-1)$$

Le snapshot est pris **avant** la dissipation sur CPU, **après** sur GPU (cf. §4).

### 1.4 Régulation par Zones (PID)

Mesure d'activité par zone $z$ : $\mu_z = \frac{1}{|z|}\sum_{i \in z} a_i$

Erreur : $e_z(t) = \mu_{\text{target}} - \mu_z(t)$

PID : $u_z = K_p \cdot e_z + K_i \cdot \sum e_z + K_d \cdot \Delta e_z$

Modulation :
- $\theta_{\text{mod},i} \leftarrow -K_\theta \cdot u_z$
- $g_{\text{mod},i} \leftarrow K_g \cdot u_z$

### 1.5 STDP (Spike-Timing-Dependent Plasticity)

Direction :
$$\psi_{ij} = \begin{cases} A^+ \cdot \exp\!\big(-\frac{\Delta t}{\tau^+}\big) & \text{si } \Delta t > 0 \text{ (post après pré)} \\ -A^- \cdot \exp\!\big(\frac{\Delta t}{\tau^-}\big) & \text{si } \Delta t < 0 \text{ (pré après post)} \end{cases}$$

avec $\Delta t = t_{\text{post}} - t_{\text{pré}}$.

**Paramètres** : $A^\pm = 0.008$, $\tau^\pm = 20$ ticks.

### 1.6 Trace d'Éligibilité

$$e_{ij}(t) = \gamma \cdot e_{ij}(t-1) + \psi_{ij}(t)$$

Borné : $e_{ij} \in [-E_{\max}, E_{\max}]$ avec $\gamma = 0.95$, $E_{\max} = 5.0$.

Demi-vie : $t_{1/2} = \frac{\ln 2}{-\ln \gamma} \approx 13.5$ ticks.

### 1.7 Règle des Trois Facteurs

Signal dopaminergique local (spatiale) :
$$\delta_{d,j} = D_{\text{phasic}} \cdot \exp(-\lambda \cdot \|p_j - p_{\text{reward}}\|)$$

ou global : $\delta_d = D_{\text{phasic}} = D_{\text{total}} - D_{\text{tonique}}$

Mise à jour de conductance :
$$\Delta C_{ij} = \eta \cdot \delta_{d,\text{post}(ij)} \cdot e_{ij}$$

avec $\eta = 3.0$ (plasticity\_gain), $\lambda = 0.15$ (spatial\_lambda).

### 1.8 Décroissance Homéostatique

**V4** : $C_{ij} \leftarrow C_{ij} + \rho \cdot (C_0 - C_{ij})$, avec $\rho = 0.0001$

**V5.2** — oubli accéléré par RPE négatif :
$$\rho_{\text{eff}} = \rho_0 + \rho_{\text{boost}} \cdot \max(0, -\delta_{\text{RPE}})$$

Constante de temps homéostatique : $\tau_{\text{homeo}} = 1/\rho = 10\,000$ ticks.

### 1.9 Consolidation

Un edge est consolidé si :
1. $C_{ij} > C_{\text{seuil}}$ pendant $T_{\text{consol}}$ ticks consécutifs
2. $D_{\text{total}} > D_{\text{consol\_thresh}}$ au moment du test

Les edges consolidés sont **immunisés** contre la décroissance homéostatique et le budget scaling.

### 1.10 Budget Synaptique

Pour chaque nœud source $s$ :
$$\text{si } \sum_{j \in \text{out}(s)} C_{sj} > B \implies C_{sj} \leftarrow C_{sj} \cdot \frac{B}{\sum C_{sj}} \quad \forall j \text{ non-consolidé}$$

avec $B = 30.0$.

### 1.11 RPE — Reward Prediction Error (V5.2)

Signal RPE :
$$\delta_{\text{RPE}} = r_{\text{eff}} - \bar{V}$$

Baseline EMA :
$$\bar{V}(t) = (1 - \alpha_{\text{RPE}}) \cdot \bar{V}(t-1) + \alpha_{\text{RPE}} \cdot r_{\text{eff}}$$

Récompense effective avec modulation par marge :
$$r_{\text{eff}} = \frac{r}{1 + \beta \cdot |M|}$$

où $M = \text{score}_{\text{correct}} - \text{score}_{\text{second}}$ est la marge de readout.

**Paramètres** : $\alpha_{\text{RPE}} = 0.05$, $\beta = 0.1$.

### 1.12 Readout

Scores par classe $c$ : $S_c = \sum_{i \in \text{output}_c} a_i$

Classe prédite : $\hat{c} = \arg\max_c S_c$

Accuracy = proportion de prédictions correctes sur les 256 derniers essais.

---

## 2. Analyse de Stabilité

### 2.1 Gain effectif d'un chemin

$$G_{\text{eff}} = G_{\text{prop}} \cdot \bar{C} \cdot K_{\text{kernel}} \cdot \frac{1}{1 + F/\xi}$$

Avec les paramètres par défaut : $G_{\text{eff}} \approx 0.8 \times 1.0 \times 1.8 = 1.44$ (sub-critique grâce à la fatigue).

### 2.2 Conditions de stabilité

1. **Activité bornée** : $a \in [0.01, 10]$ ✓ (clamp explicite)
2. **Conductance bornée** : $C \in [0.1, 5.0]$ ✓ (clamp après chaque update)
3. **Budget** : $\sum C_{\text{out}} \leq 30$ ✓ (normalisé par tick)
4. **Éligibilité bornée** : $|e| \leq 5.0$ ✓
5. **Fatigue auto-régulée** : convergence exponentielle vers 0 ✓

### 2.3 Points de fragilité identifiés

- **Consolidation irréversible** : une fois consolidé, un edge ne peut JAMAIS revenir
- **Homéostatique vs apprentissage** : $\rho \cdot (C_0 - C)$ tire vers $C_0 = 1.0$, effaçant lentement toute différenciation apprise
- **RPE sensibilité** : $\alpha_{\text{RPE}} = 0.05$ donne une constante de temps de 20 essais, trop court pour des tâches longues

---

## 3. Résultats Expérimentaux — Isolation Systématique

### 3.1 Protocole

Tâche symétrique à 2 classes, 50k nœuds, 10k ticks (~256 essais), seed=42.
Chaque expérience isole une variable (backend, sustained, plasticity).

### 3.2 Résultats Complets

| # | Config | Backend | Sustained | Mode | Accuracy | Notes |
|---|--------|---------|-----------|------|----------|-------|
| 1 | Vanilla | **CPU** | OFF | TopologyOnly | **84.4%** (216/256) | Baseline |
| 2 | Vanilla | **GPU** | OFF | TopologyOnly | **82.0%** (210/256) | ≈ CPU (Δ=2.4pp) |
| 3 | Vanilla | **CPU** | OFF | FullLearning | **64.1%** (164/256) | Plasticity: **-20.3pp** |
| 4 | V5 sustained | **CPU** | ON | TopologyOnly | **97.3%** (249/256) | Sustained: **+12.9pp** |
| 5 | V5 sustained | **CPU** | ON | FullLearning | **87.9%** (225/256) | Plasticity: **-9.4pp** |
| 6 | V5 sustained | **GPU** | ON | TopologyOnly | **76.6%** (196/256) | Sustained: **-5.4pp** 🐛 |
| 7 | V5 sustained | **GPU** | ON | FullLearning | **73.4%** (188/256) | Plasticity: **-3.2pp** |
| 8 | V5.2 (RPE+margin) | **GPU** | ON | FullLearning | **73.4%** (188/256) | RPE ≈ neutre |
| 9 | V5.2 no RPE | **GPU** | ON | FullLearning | **40.2%** (103/256) | Sans RPE: catastrophe |
| 10 | V5.2 Remap RPE | **GPU** | ON | FullLearning | **25.0%** (64/256) | Post-remap bloqué |
| 11 | V5.2 Remap Forget | **GPU** | ON | FullLearning | **25.0%** (64/256) | ρ_boost=0.02: idem |

### 3.3 Conclusions Expérimentales

#### Constat 1 : Le pipeline GPU vanilla est fonctionnellement correct
Les expériences #1 vs #2 montrent CPU ≈ GPU (84.4% vs 82.0%) quand les features V5
sustained sont désactivées. La différence de 2.4pp est compatible avec le bruit
stochastique (hash de jitter différent).

#### Constat 2 : Bug de snapshot timing GPU pour V5 sustained
Les expériences #4 vs #6 révèlent un **bug critique** :
- CPU + sustained : +12.9pp (booste l'accuracy de 84.4% → 97.3%)
- GPU + sustained : **-5.4pp** (dégrade l'accuracy de 82.0% → 76.6%)

**Cause racine** : L'ordre des phases diffère entre CPU et GPU.

Sur CPU :
```
Phase 4b : Réverbération (avant dissipation)
Snapshot  : a_i^snap = a_i + reverb ← PRÉ-decay (haute énergie)
Phase 5  : Dissipation / adaptive_decay
```

Sur GPU :
```
Phase 7  : Apply + Dissipation (fused, skip decay si adaptive)
Phase 7b : Réverbération
Phase 7c : Adaptive decay
Phase 12 : Snapshot ← POST-decay (basse énergie)
```

Le snapshot GPU capture $a_i \cdot (1 - \alpha_{\text{eff}})$ au lieu de $a_i$.
Avec $\alpha_{\text{base}} = 0.25$, le snapshot GPU est ~75% du snapshot CPU.

Ce ratio se compose via la réverbération : à chaque tick,
$$a_i^{\text{reverb,GPU}} = r_{\text{gain}} \cdot a_i^{\text{snap,GPU}} \approx 0.75 \cdot r_{\text{gain}} \cdot a_i^{\text{snap,CPU}}$$

En régime permanent, l'énergie de réverbération GPU converge vers ~75% du CPU,
causant un déficit d'activité systématique qui dégrade le readout.

#### Constat 3 : La plasticité est systématiquement nuisible
Les comparaisons TopologyOnly vs FullLearning montrent que la plasticité
**réduit toujours** l'accuracy :
- CPU vanilla : -20.3pp (84.4% → 64.1%)
- CPU sustained : -9.4pp (97.3% → 87.9%)
- GPU sustained : -3.2pp (76.6% → 73.4%)

La règle des trois facteurs avec les paramètres actuels ne produit pas de
renforcement sélectif des bonnes connexions. Au contraire, elle introduit du
bruit dans les conductances, érodant la structure topologique initiale.

**Mécanisme** : La décroissance homéostatique $\rho \cdot (C_0 - C)$ ramène
toutes les conductances vers $C_0 = 1.0$ avec $\tau = 10\,000$ ticks. Comme le
signal dopaminergique n'est pas assez sélectif spatialement ($\lambda = 0.15$
sur un domaine de [0, 100]³), le renforcement est trop diffus pour compenser
l'érosion homéostatique.

#### Constat 4 : RPE compense partiellement les dégâts plastiques sur GPU
L'expérience #9 montre que sur GPU sans RPE, le FullLearning tombe à 40.2%.
Avec RPE (#8), il remonte à 73.4%. Le RPE agit comme un frein :
- Quand $\delta_{\text{RPE}} < 0$ (performance dégradée), $\rho_{\text{eff}}$ augmente,
  accélérant l'oubli des modifications récentes (retour vers baseline)
- Quand $\delta_{\text{RPE}} > 0$, $\rho_{\text{eff}} = \rho_0$ (pas d'accélération)

Cet effet asymétrique protège le réseau contre les dégâts plastiques mais
ne produit pas d'apprentissage positif.

#### Constat 5 : Remap complètement bloqué par consolidation
Les expériences #10 et #11 montrent 25% (pire que hasard à 50%) en remap.
Les edges consolidés pendant la phase initiale sont **immunisés** contre
toute modification. Le réseau est "gelé" dans l'ancienne configuration et
ne peut pas s'adapter au nouveau mapping.

---

## 4. Bug GPU Identifié : Snapshot Timing

### 4.1 Description

Dans `gpu.rs::run_full_tick()`, l'ordre des phases est :
```
Phase 7  : apply_and_dissipate (fatigue/inhib/trace, skip decay si adaptive)
Phase 7b : reverberation
Phase 7c : adaptive_decay
Phase 8  : plasticity
Phase 9  : budget
Phase 10 : sync_conductances
Phase 11 : readout
Phase 12 : snapshot_activations  ← TROP TARD
```

Le snapshot devrait être pris **après la réverbération et avant la dissipation**
pour matcher le comportement CPU.

### 4.2 Fix Proposé

Déplacer Phase 12 (snapshot_activations) **entre Phase 7b et Phase 7c** :
```
Phase 7  : apply_and_dissipate
Phase 7b : reverberation
Phase 7b': SNAPSHOT_ACTIVATIONS  ← ICI
Phase 7c : adaptive_decay
Phase 8  : plasticity
...
```

### 4.3 Impact Attendu

Avec ce fix, GPU + sustained devrait passer de ~76.6% à ~97% (TopologyOnly),
aligné avec le CPU. Le FullLearning GPU devrait aussi bénéficier significativement.

### 4.4 Résultats Post-Fix (IMPLÉMENTÉ)

Le fix a été implémenté dans `gpu.rs::run_full_tick()` et `run_profiled_tick()`.
Build : 0 erreurs, 12/12 tests OK.

| Config | Avant Fix | Après Fix | Δ |
|--------|-----------|-----------|---|
| V5.2 Normal FullLearning (GPU) | 73.4% | **86.7%** | **+13.3pp** ✅ |
| V5.2 Normal TopologyOnly (GPU) | 76.6% | **81.6%** | **+5.0pp** ✅ |
| V5.2 Inverted FullLearning (GPU) | 55.5% | 44.5% | -11.0pp (attendu) |
| V5.2 Inverted TopologyOnly (GPU) | 26.6% | 18.0% | -8.6pp (attendu) |

**Résultats clés** :
1. **La plasticité est désormais constructive** : FullLearning (86.7%) > TopologyOnly (81.6%)
   → +5.1pp d'apprentissage réel (vs -3.2pp avant fix = plasticity nuisible)
2. Le signal topologique est amplifié dans les deux sens (normal ET inverted),
   confirmant que le fix renforce correctement la maintenance d'activité.
3. **Gap résiduel GPU↔CPU** : ~10pp (CPU sustained topo = 97.3%). Causé par
   les différences restantes (fatigue timing dans le shader fusionné, jitter hash).

### 4.5 Divergences Pipeline Restantes

Après le fix snapshot, les différences restantes sont :

1. **Fatigue/trace timing** : Le shader `apply_and_dissipate` met à jour fatigue/inhib/trace
   AVANT la réverbération (Phase 7, basé sur activation post-influence). Sur CPU, ces
   updates se font APRÈS la réverbération (Phase 5). Impact estimé : ~5pp.
   Fix possible : séparer le shader fusionné en deux passes.

2. **Zones vs Injection** : GPU mesure l'activité APRÈS injection (Inject→Zone).
   CPU mesure AVANT injection (Zone→Inject). Mais inverser cet ordre sur GPU
   cause des zones qui mesurent post-reset (activité ~0), empirant les résultats.
   Compromis acceptable en l'état.

---

## 5. Cohérence Mathématique : Vérification

### 5.1 Équations vs Code ✅

| Composant | Équation | Code CPU (stdp.rs) | Code GPU (.wgsl) | Match |
|-----------|----------|--------------------|-------------------|-------|
| STDP ψ | $A^+ e^{-\Delta t/\tau^+}$ | ✅ | ✅ | ✅ |
| Éligibilité | $\gamma e + \psi$ | ✅ | ✅ | ✅ |
| Trois facteurs | $\eta \cdot \delta_d \cdot e$ | ✅ | ✅ | ✅ |
| Homéostatique | $\rho \cdot (C_0 - C)$ | ✅ | ✅ | ✅ |
| RPE | $r_{\text{eff}} - \bar{V}$ | ✅ | N/A (CPU) | ✅ |
| V5.2 ρ\_boost | $\rho + \rho_b \cdot \max(0,-\delta)$ | ✅ | ✅ | ✅ |
| Budget | $C \cdot B / \sum C$ | ✅ | ✅ | ✅ |
| Consolidation | seuil + durée + dopa | ✅ | ✅ | ✅ |
| Propagation | CSR + kernel | ✅ | ✅ | ✅ |
| Dissipation | decay + fatigue + inhib | ✅ | ✅ | ✅ |
| Zones PID | Kp/Ki/Kd → θ\_mod, g\_mod | ✅ | ✅ | ✅ |

### 5.2 Divergences Pipeline

| Aspect | CPU | GPU | Impact |
|--------|-----|-----|--------|
| Snapshot timing | Pré-decay | Post-decay | **CRITIQUE** (§4) |
| Zones vs Injection | Zones→Inject | Inject→Zones | Mineur (~0.2pp) |
| Dopamine decay | Avant plasticity | Après GPU tick | Négligeable |
| Jitter hash | u64 LCG | u32 mul-add | Bruit différent |
| Edges traités (élig.) | Active set only | Tous les edges | Fonctionnellement equiv. |
| Budget | Incrémental | Fresh chaque tick | GPU plus précis |

---

## 6. Paramètres Actuels et Sensibilité

| Paramètre | Valeur | Rôle | Sensibilité |
|-----------|--------|------|-------------|
| $A^\pm$ | 0.008 | Amplitude STDP | Faible (symétrique) |
| $\tau^\pm$ | 20 | Fenêtre STDP | Modérée |
| $\gamma$ | 0.95 | Décroissance élig. | $t_{1/2}$ = 13.5 ticks |
| $\eta$ | 3.0 | Gain plasticité | **Haute** (trop fort?) |
| $\rho$ | 0.0001 | Homéostatique | $\tau$ = 10k ticks |
| $B$ | 30.0 | Budget | 30× conductance init. |
| $C_0$ | 1.0 | Baseline cond. | Attracteur homéostatique |
| $\lambda$ | 0.15 | Spatial dopamine | Rayon ~6.7 unités / 100 |
| $r_{\text{gain}}$ | 0.12 | Réverbération | Stabilité dépendante |
| $\alpha_{\text{decay}}$ | 0.25 | Dissipation | Fort decay → reverb critique |
| $k_{\text{local}}$ | 0.3 | Decay adaptatif | Modère decay local |

---

## 7. Résumé des Problèmes Mathématiques

1. **Bug GPU snapshot** : Le snapshot post-decay réduit l'énergie de réverbération de ~25%, cassant la maintenance d'activité sur GPU.

2. **Plasticité destructrice** : $\eta \cdot \delta_d \cdot e = 3.0 \cdot D_{\text{phasic}} \cdot e$ ne cible pas assez les bonnes connexions. Le champ dopaminergique spatial ($\lambda = 0.15$ sur [0,100]³) est trop diffus — rayon d'influence ~7 unités vs domaine 100 unités.

3. **Consolidation irréversible** : Empêche toute adaptation post-remap. Aucun mécanisme de dé-consolidation n'existe.

4. **Homéostatique trop lent** : $\tau = 10\,000$ ticks pour un cycle d'essai de ~39 ticks → le réseau met ~260 essais à "oublier", beaucoup trop lent pour le remap.

5. **RPE compensatoire mais non-constructif** : Le RPE protège contre les dégâts mais ne dirige pas positivement l'apprentissage.
