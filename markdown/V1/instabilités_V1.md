# Instabilités et limites V1 — Pistes pour V2+

## Résumé

Le système V1 produit des **ondes transitoires unidirectionnelles** (stimulus → propagation → extinction → repos). Il ne supporte ni oscillations entretenues, ni ondes sinusoïdales, ni régulation locale de l'activité. Ce document inventorie les instabilités identifiées, les mécanismes manquants, et les pistes architecturales pour les versions futures.

---

## 1. Instabilités identifiées

### 1.1 Saturation → période réfractaire prolongée

**Problème** : Quand un groupe de nœuds atteint $a = 10$ (activation maximale), la fatigue s'accumule rapidement :

$$f(t) \to \frac{\gamma_f \cdot a_{\max}}{\rho_f} = \frac{0{,}20 \times 10}{0{,}05} = 40 \quad \text{(clampé à 10)}$$

Le seuil effectif monte à $\theta^{\text{eff}} \approx 8{,}9$ après ~4 ticks, coupant la propagation. Mais ensuite :

- La fatigue ne décroît qu'à 5%/tick (demi-vie ~13 ticks).
- Pour qu'un stimulus $I = 1$ réactive un nœud, il faut $f < 0{,}8$, soit **~46 ticks d'attente**.
- Le système est fonctionnellement **mort** pendant cette période réfractaire.

**Impact** : Une saturation locale crée un trou noir temporel de ~50 ticks dans la zone concernée.

### 1.2 Absence d'équilibre actif

**Problème** : Démontré mathématiquement (maths.md §10), le système ne possède **aucun point fixe actif stable**. Les rétroactions négatives (fatigue + inhibition) augmentent le seuil effectif de tout nœud actif de façon monotone :

$$\theta_i^{\text{eff}} \to \frac{\theta + \gamma_f a^*/\rho_f + \gamma_h/\delta_h}{\varepsilon^*} \geq 2{,}8$$

**Conséquence** : Toute activité finit par s'éteindre quoi qu'il arrive. Le seul état stationnaire est le repos $a^* = a_{\min}$.

### 1.3 Conductance en dérive vers le plancher

**Problème** : En l'absence d'activité, toutes les arêtes subissent un affaiblissement constant de $\eta_- \cdot p$ par tick. Les conductances convergent asymptotiquement vers $w_{\min} = 0{,}1$.

$$w_{ij}(t) \to w_{\min} \quad \text{en } \sim\frac{w_0 - w_{\min}}{2\eta_-} \approx 250 \text{ ticks}$$

**Impact** : Toute mémoire structurelle (chemins renforcés par l'apprentissage) est éphémère. Sans stimulation continue, le réseau **oublie tout** en ~250 ticks.

### 1.4 Asymétrie temporelle absolue

**Problème** : Toutes les équations de dissipation sont des contractions ($\times(1-r)$ avec $r > 0$). Il n'existe **aucun mécanisme d'augmentation spontanée** d'activation, fatigue, inhibition ou conductance sans stimulus externe ou propagation active.

**Conséquence** : Le système ne peut pas générer d'activité endogène. C'est un système strictement **stimulus-réponse**.

### 1.5 Uniformité des nœuds

**Problème** : Tous les nœuds sont gouvernés par les mêmes équations et les mêmes paramètres. La seule différenciation vient des positions spatiales et des traces mémoire accumulées.

**Impact** : Pas de spécialisation fonctionnelle, pas de hiérarchie, pas de rôles distincts (capteurs, intégrateurs, régulateurs).

---

## 2. Dynamiques impossibles en V1

### 2.1 Oscillations entretenues / ondes sinusoïdales

**Requis** : Un mécanisme de rétroaction positive **contrôlée** qui cycliquement monte puis redescend l'activation.

**Pourquoi c'est impossible en V1** :
- La fatigue est monotone croissante pendant l'activité → pas de phase descendante contrôlée.
- Le recovery de fatigue ($\rho_f = 0{,}05$) est trop lent par rapport au gain ($\gamma_f = 0{,}20$) → seuil effectif ne redescend pas avant l'extinction totale.
- Le seul cycle possible est : activation → saturation → extinction forcée → recharge passive → mais ce n'est pas une sinusoïde, c'est un **relaxation oscillator fortement amorti**.

### 2.2 Ondes stationnaires

**Requis** : Des patterns d'interférence spatiale stables.

**Pourquoi c'est impossible** : L'onde ne se réfléchit pas aux bords (pas de bords) et ne peut pas se maintenir (pas de source d'énergie interne). Toute onde est simplement absorbée par la dissipation.

### 2.3 Rythmes intrinsèques

**Requis** : Qu'un nœud ou un groupe de nœuds puisse osciller à une fréquence propre sans stimulus externe.

**Pourquoi c'est impossible** : Sans stimulus, $a_i(t) \to a_{\min}$ monotonement. Il n'y a aucun mécanisme de **réinjection** d'énergie dans le système.

### 2.4 Régulation locale (homéostasie)

**Requis** : Maintenir une activité cible dans une zone malgré les perturbations.

**Pourquoi c'est impossible** : Aucun nœud n'a de "consigne" ($a_{\text{target}}$) ni de capacité à ajuster ses paramètres locaux en réponse à un écart.

---

## 3. Pistes architecturales pour V2+

### 3.1 Neurones de contrôle — Régulateurs pseudo-PID

**Concept** : Introduire une classe spéciale de nœuds (**neurones de contrôle**) qui ne propagent pas directement mais **régulent** l'activité de leur zone.

#### Architecture proposée

```
┌─────────────────────────┐
│ Neurone de contrôle (NC) │
│                          │
│  Entrée : ā_zone(t)     │  ← activité moyenne de sa zone
│  Consigne : a_target     │  ← paramètre configurable
│  Erreur : e = a_target - ā_zone │
│                          │
│  P : u_P = K_P · e       │
│  I : u_I += K_I · e · Δt │
│  D : u_D = K_D · (e - e_prev) / Δt │
│                          │
│  Sortie : u = u_P + u_I + u_D │
│                          │
│  Action : injecte u dans │
│  les nœuds de sa zone    │
└─────────────────────────┘
```

#### Fonctionnement PID
- **P (Proportionnel)** : si l'activité est en dessous de la consigne, injecter de l'énergie proportionnellement à l'écart. Si au-dessus, injecter une inhibition.
- **I (Intégral)** : compense les erreurs persistantes (drift de conductance, fatigue résiduelle).
- **D (Dérivé)** : anticipe les variations rapides (front d'onde arrivant → freiner; extinction → booster).

#### Capacités débloquées
- **Oscillations sinusoïdales** : la consigne elle-même peut être modulée sinusoïdalement : $a_{\text{target}}(t) = A \sin(2\pi f t + \phi) + a_0$. Le PID force les nœuds de la zone à suivre cette consigne.
- **Fréquences locales** : chaque NC peut avoir sa propre fréquence → superposition d'ondes, battements, harmoniques.
- **Stabilité intrinsèque** : le terme I élimine l'erreur statique, le terme D amortit les oscillations parasites.

### 3.2 Zones et partitionnement spatial

Pour que les NC fonctionnent, il faut **partitionner** l'espace en zones :
- Partitionnement par Voronoï autour des positions des NC.
- Ou par rayon fixe ($r_{\text{zone}}$) autour de chaque NC.
- Ou par clustering (K-means sur les positions 3D).

Chaque nœud standard appartient à **exactement un NC** (ou à plusieurs avec pondération par distance).

### 3.3 Cycle de vie enrichi d'un tick (V2)

```
Phase 0 : Mesure      — chaque NC calcule ā_zone
Phase 1 : Régulation  — chaque NC calcule son u_PID et l'injecte
Phase 2 : Stimulus    — injections externes
Phase 3 : Propagation — comme V1
Phase 4 : Dissipation — comme V1
Phase 5 : Plasticité  — comme V1
```

La régulation agit **avant** la propagation, permettant au NC de préparer le terrain.

### 3.4 Mémoire structurelle persistante

**Problème V1** : la conductance dérive vers $w_{\min}$ sans activité.

**Solution V2** : introduire un **seuil de consolidation** pour les arêtes :

$$\text{si } w_{ij} > w_{\text{consolidation}} \text{ pendant } T_{\text{consol}} \text{ ticks} \implies \text{decay désactivé pour cette arête}$$

Ou un taux de decay adaptatif :

$$\eta_-^{\text{eff}}(w) = \eta_- \cdot \max(0, 1 - w/w_{\text{protect}})$$

Les arêtes très renforcées deviennent quasi-permanentes.

### 3.5 Nœuds pacemaker (alternative aux NC)

Des nœuds qui oscillent **intrinsèquement** sans PID :

$$a_i(t+1) = a_i(t) + A_{\text{pace}} \sin(2\pi f_i t)$$

Plus simple qu'un NC-PID mais sans capacité de régulation. Peut être un premier pas vers V2.

### 3.6 Rétro-inhibition non-linéaire

Remplacer la fatigue linéaire par une dynamique non-linéaire permettant des cycles :

$$f_i(t+1) = f_i(t) + \gamma_f \frac{a_i^2}{a_i^2 + K_f^2} - \rho_f \cdot f_i$$

Le terme de Hill ($a^2 / (a^2 + K_f^2)$) sature → la fatigue ne peut plus monter indéfiniment, ce qui permet des oscillations de relaxation.

---

## 4. Résumé des instabilités → solutions

| # | Instabilité V1 | Impact | Solution V2 proposée |
|---|---|---|---|
| I1 | Saturation → réfractaire ~50 ticks | Zone morte prolongée | Fatigue non-linéaire (Hill) ou NC-PID |
| I2 | Pas d'équilibre actif | Toute activité s'éteint | NC-PID avec consigne > 0 |
| I3 | Conductance → $w_{\min}$ | Oubli total en ~250 ticks | Consolidation / decay adaptatif |
| I4 | Pas d'activité endogène | Système purement réactif | Pacemakers ou NC avec oscillateur |
| I5 | Nœuds uniformes | Pas de spécialisation | Types de nœuds (standard, contrôle, pacemaker) |
| I6 | Pas d'oscillations | Pas d'ondes sinusoïdales | NC-PID avec consigne sinusoïdale |
| I7 | Pas de régulation locale | Pas d'homéostasie | NC-PID par zone |

---

## 5. Priorités suggérées pour V2

```
Priorité 1 (fondation) :
  ├── Types de nœuds (standard / contrôle)
  ├── Partitionnement spatial en zones
  └── Mesure d'activité moyenne par zone

Priorité 2 (régulation) :
  ├── Régulateur PID dans les neurones de contrôle
  ├── Consigne configurable (constante puis sinusoïdale)
  └── Injection régulée dans la zone

Priorité 3 (mémoire) :
  ├── Consolidation des arêtes fortement renforcées
  └── Decay adaptatif des conductances

Priorité 4 (dynamiques riches) :
  ├── Consignes sinusoïdales multi-fréquences
  ├── Couplage entre NC (synchronisation de phase)
  └── Fatigue non-linéaire (Hill)
```

---

## 6. Condition nécessaire pour les ondes sinusoïdales

Pour qu'une onde sinusoïdale $a_i(t) = A\sin(\omega t + \phi_i) + a_0$ soit une solution (même approximative) du système, il faut :

1. **Source d'énergie interne** : un mécanisme (NC-PID ou pacemaker) qui injecte $+\Delta a$ quand $a < a_{\text{target}}$ et $-\Delta a$ quand $a > a_{\text{target}}$.

2. **Fatigue réversible** : la fatigue doit pouvoir **descendre aussi vite qu'elle monte**, sinon le seuil effectif augmente monotonement et tue l'oscillation en quelques cycles.

3. **Couplage spatial cohérent** : pour qu'une onde se propage comme une sinusoïde spatiale, le déphasage $\phi_i$ entre nœuds voisins doit être déterminé par la distance : $\phi_i = \omega d_i / v$ (vitesse de propagation $v$). Cela requiert un noyau de propagation qui respecte la causalité temporelle (délai proportionnel à la distance — actuellement absent en V1 où la propagation est instantanée).

4. **Absence de saturation** : l'amplitude $A$ doit rester dans la zone linéaire du système (loin des clamps à 0 et 10).

**Aucune de ces 4 conditions n'est satisfaite en V1.** Le NC-PID adresse les conditions 1 et partiellement 2. La condition 3 nécessite un mécanisme de **délai de propagation** ($\tau_{ij} \propto d_{ij}$). La condition 4 requiert un dimensionnement soigneux de l'amplitude et des gains.
