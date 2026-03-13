# Formalisation mathématique du système — V1

## 1. Notation et conventions

Le système évolue en **temps discret**. Un pas de temps est noté un **tick** $t \in \mathbb{N}$.

### 1.1 Graphe

Le substrat est un graphe orienté $G = (V, E)$ plongé dans $\mathbb{R}^3$ :

- $V = \{1, \dots, N\}$ — ensemble des nœuds
- $E \subseteq V \times V$ — ensemble des arêtes orientées
- $\mathcal{N}(i) = \{j : (i, j) \in E\}$ — voisins sortants du nœud $i$
- $\mathcal{N}^{-}(i) = \{j : (j, i) \in E\}$ — voisins entrants du nœud $i$
- $\mathbf{p}_i \in \mathbb{R}^3$ — position spatiale du nœud $i$
- $d_{ij} = \|\mathbf{p}_i - \mathbf{p}_j\|_2$ — distance euclidienne (constante)

### 1.2 Variables d'état par nœud

Pour chaque nœud $i$ au tick $t$ :

| Symbole | Variable | Domaine |
|---|---|---|
| $a_i(t)$ | activation | $[a_{\min}, 10]$ |
| $\theta_i$ | seuil de base | $\mathbb{R}^+$ (constant) |
| $f_i(t)$ | fatigue | $[0, 10]$ |
| $h_i(t)$ | inhibition | $[0, 10]$ |
| $m_i(t)$ | trace mémoire | $[0, 100]$ |
| $\varepsilon_i(t)$ | excitabilité | $\mathbb{R}^+$ |

### 1.3 Variables d'état par arête

Pour chaque arête $(i \to j)$ au tick $t$ :

| Symbole | Variable | Domaine |
|---|---|---|
| $w_{ij}(t)$ | conductance | $[w_{\min}, w_{\max}]$ |
| $c_{ij}(t)$ | trace de co-activation | $[0, 10]$ |
| $d_{ij}$ | distance spatiale | $\mathbb{R}^+$ (constant) |
| $p_{ij}$ | coefficient de plasticité | $\mathbb{R}^+$ (constant) |

### 1.4 Paramètres globaux

| Symbole | Paramètre | Section |
|---|---|---|
| $g$ | gain de propagation | Propagation |
| $\lambda$ | décroissance spatiale | Propagation |
| $\alpha$ | taux de decay de l'activation | Dissipation |
| $a_{\min}$ | activation minimale (potentiel de repos) | Dissipation |
| $J$ | amplitude du jitter sur le decay | Dissipation |
| $\gamma_f$ | gain de fatigue | Dissipation |
| $\rho_f$ | taux de récupération de la fatigue | Dissipation |
| $\gamma_h$ | gain d'inhibition | Dissipation |
| $\delta_h$ | taux de decay de l'inhibition | Dissipation |
| $\gamma_m$ | gain de trace mémoire | Dissipation |
| $\delta_m$ | taux de decay de la trace | Dissipation |
| $\eta_+$ | taux de renforcement | Plasticité |
| $\eta_-$ | taux d'affaiblissement | Plasticité |
| $c_\theta$ | seuil de co-activation | Plasticité |
| $\delta_c$ | decay de la trace de co-activation | Plasticité |

---

## 2. Quantités dérivées

### 2.1 Seuil effectif

$$\theta_i^{\text{eff}}(t) = \frac{\theta_i + f_i(t) + h_i(t)}{\max(\varepsilon_i(t),\; 0{,}01)}$$

### 2.2 Prédicat d'activation

$$\sigma_i(t) = \mathbb{1}\!\left[a_i(t) > \theta_i^{\text{eff}}(t)\right]$$

Un nœud est **actif** si et seulement si $\sigma_i(t) = 1$.

---

## 3. Séquence d'un tick

Chaque tick $t \to t+1$ exécute **4 phases séquentielles** dans cet ordre strict. L'ordre est fondamental car chaque phase lit l'état produit par la phase précédente.

$$\boxed{t \xrightarrow{\text{1. Stimulus}} \xrightarrow{\text{2. Propagation}} \xrightarrow{\text{3. Dissipation}} \xrightarrow{\text{4. Plasticité}} t+1}$$

Nous notons $x^{(k)}$ l'état de la variable $x$ après la phase $k$ du tick.

---

## 4. Phase 1 — Injection de stimulus

Pour chaque stimulus $s$ prévu au tick $t$, ciblant le nœud $i_s$ avec intensité $I_s$ :

$$a_i^{(1)} = a_i(t) + \sum_{s \,:\, \text{actif}(s,t)} I_s \cdot \mathbb{1}[i = i_s]$$

Condition d'activité d'un stimulus $s$ au tick $t$ :

$$\text{actif}(s, t) = \begin{cases} 1 & \text{si } t_{\text{start}} \leq t < t_{\text{end}} \text{ et } (t - t_{\text{start}}) \bmod R_s = 0 \\ 0 & \text{sinon} \end{cases}$$

avec $R_s$ l'intervalle de répétition ($R_s = 0$ → actif à chaque tick de la plage).

---

## 5. Phase 2 — Propagation

### 5.1 Calcul des influences

Le noyau de propagation $K$ est défini par :

$$K_{\text{exp}}(d) = e^{-\lambda \, d}$$

$$K_{\text{gauss}}(d) = e^{-\frac{1}{2}(\lambda \, d)^2}$$

L'influence totale reçue par le nœud $i$ :

$$\varphi_i = \sum_{j \in \mathcal{N}^{-}(i)} \sigma_j^{(1)} \cdot a_j^{(1)} \cdot w_{ji}(t) \cdot K(d_{ji}) \cdot g$$

Seuls les nœuds actifs ($\sigma_j = 1$) propagent.

### 5.2 Application des influences

$$a_i^{(2)} = \min\!\Big(a_i^{(1)} + \varphi_i,\; 10\Big)$$

Le bornage supérieur à 10 empêche l'explosion d'activation.

---

## 6. Phase 3 — Dissipation

Les sous-étapes sont exécutées **séquentiellement dans cet ordre** pour chaque nœud $i$. Toutes sont appliquées **nœud par nœud** (pas en parallèle global).

Le prédicat d'activité $\sigma_i^{(2)}$ est évalué par rapport à l'état post-propagation.

### 6.1 Fatigue

$$f_i^{(3)} = \begin{cases}
\Big(f_i(t) + \gamma_f \cdot a_i^{(2)}\Big)(1 - \rho_f) & \text{si } \sigma_i^{(2)} = 1 \\[4pt]
f_i(t) \cdot (1 - \rho_f) & \text{sinon}
\end{cases}$$

$$f_i^{(3)} = \text{clamp}\!\left(f_i^{(3)},\; 0,\; 10\right)$$

### 6.2 Inhibition

$$h_i^{(3)} = \begin{cases}
\Big(h_i(t) + \gamma_h\Big)(1 - \delta_h) & \text{si } \sigma_i^{(2)} = 1 \\[4pt]
h_i(t) \cdot (1 - \delta_h) & \text{sinon}
\end{cases}$$

$$h_i^{(3)} = \text{clamp}\!\left(h_i^{(3)},\; 0,\; 10\right)$$

### 6.3 Trace mémoire

$$m_i^{(3)} = \begin{cases}
\Big(m_i(t) + \gamma_m \cdot a_i^{(2)}\Big)(1 - \delta_m) & \text{si } \sigma_i^{(2)} = 1 \\[4pt]
m_i(t) \cdot (1 - \delta_m) & \text{sinon}
\end{cases}$$

$$m_i^{(3)} = \text{clamp}\!\left(m_i^{(3)},\; 0,\; 100\right)$$

### 6.4 Excitabilité

$$\varepsilon_i^{(3)} = 1 + 0{,}1 \cdot \min(m_i^{(3)},\; 5)$$

L'excitabilité est une fonction déterministe de la trace. Domaine résultant : $\varepsilon \in [1{,}0\;;\; 1{,}5]$.

### 6.5 Decay de l'activation (avec jitter)

Le taux effectif de decay pour chaque nœud $i$ à chaque tick :

$$\xi_i(t) \sim \mathcal{U}(-J, +J)$$

$$\alpha_i^{\text{eff}}(t) = \text{clamp}\!\left(\alpha \cdot (1 + \xi_i(t)),\; 0,\; 1\right)$$

$$a_i^{(3)} = \max\!\Big(a_i^{(2)} \cdot \left(1 - \alpha_i^{\text{eff}}(t)\right),\; a_{\min}\Big)$$

Quand $J = 0$ : $\alpha_i^{\text{eff}} = \alpha$ (comportement V0 déterministe).

---

## 7. Phase 4 — Plasticité

Pour chaque arête $(i \to j) \in E$ :

### 7.1 Co-activation

$$c_{ij}^{(4)} = \begin{cases}
\min\!\Big(c_{ij}(t) + \min(a_i^{(2)}, 1) \cdot \min(a_j^{(2)}, 1),\; 10\Big) & \text{si } a_i^{(2)} > 0{,}1 \text{ et } a_j^{(2)} > 0{,}1 \\[4pt]
c_{ij}(t) & \text{sinon}
\end{cases}$$

### 7.2 Decay de la co-activation

$$c_{ij}^{(4)} \leftarrow c_{ij}^{(4)} \cdot (1 - \delta_c)$$

### 7.3 Mise à jour de la conductance (règle Hebbienne)

$$\Delta w_{ij} = \begin{cases}
+\,\eta_+ \cdot p_{ij} \cdot \left(c_{ij}^{(4)} - c_\theta\right) & \text{si } c_{ij}^{(4)} > c_\theta \\[4pt]
-\,\eta_- \cdot p_{ij} & \text{sinon}
\end{cases}$$

$$w_{ij}^{(4)} = \text{clamp}\!\left(w_{ij}(t) + \Delta w_{ij},\; w_{\min},\; w_{\max}\right)$$

---

## 8. Équations de transition complètes

En résumant, les variables d'état au tick $t+1$ en fonction du tick $t$ (sans stimulus) :

$$\boxed{a_i(t\!+\!1) = \max\!\left(\min\!\left(a_i(t) + \varphi_i(t),\; 10\right) \cdot (1 - \alpha_i^{\text{eff}}),\; a_{\min}\right)}$$

$$\boxed{f_i(t\!+\!1) = \left(f_i(t) + \gamma_f \cdot a_i^{(2)}(t) \cdot \sigma_i^{(2)}(t)\right)(1 - \rho_f)}$$

$$\boxed{h_i(t\!+\!1) = \left(h_i(t) + \gamma_h \cdot \sigma_i^{(2)}(t)\right)(1 - \delta_h)}$$

$$\boxed{m_i(t\!+\!1) = \left(m_i(t) + \gamma_m \cdot a_i^{(2)}(t) \cdot \sigma_i^{(2)}(t)\right)(1 - \delta_m)}$$

$$\boxed{\varepsilon_i(t\!+\!1) = 1 + 0{,}1 \cdot \min(m_i(t\!+\!1),\; 5)}$$

$$\boxed{w_{ij}(t\!+\!1) = \text{clamp}\!\left(w_{ij}(t) + \Delta w_{ij}(t),\; w_{\min},\; w_{\max}\right)}$$

---

## 9. Analyse d'équilibre

### 9.1 Définition

Un **point d'équilibre** (stase) est un état $\mathbf{x}^* = (a^*, f^*, h^*, m^*, \varepsilon^*, w^*, c^*)$ tel que :

$$\mathbf{x}^*(t+1) = \mathbf{x}^*(t) = \mathbf{x}^*$$

En l'absence de stimulus externe ($I_s = 0$).

### 9.2 Équilibre trivial (repos)

**Hypothèse** : $\sigma_i = 0$ pour tout $i$ (aucun nœud actif).

Sous cette hypothèse, toutes les influences sont nulles ($\varphi_i = 0$), donc les dynamiques se découplent et deviennent des récurrences linéaires indépendantes.

**Activation** : $a_i(t+1) = \max(a_i(t) \cdot (1 - \alpha), a_{\min})$

La suite $a_i(t)$ est décroissante (pour $\alpha \in (0,1)$) et bornée inférieurement par $a_{\min}$.
Par convergence monotone :

$$\boxed{a_i^* = a_{\min}}$$

**Fatigue** : $f_i(t+1) = f_i(t)(1-\rho_f)$

$$\boxed{f_i^* = 0}$$

**Inhibition** : $h_i(t+1) = h_i(t)(1-\delta_h)$

$$\boxed{h_i^* = 0}$$

**Trace mémoire** : $m_i(t+1) = m_i(t)(1 - \delta_m)$

$$\boxed{m_i^* = 0}$$

**Excitabilité** : $\varepsilon_i^* = 1 + 0{,}1 \cdot \min(0, 5) = 1$

$$\boxed{\varepsilon_i^* = 1}$$

**Co-activité** : pas de co-activation → $c_{ij}(t+1) = c_{ij}(t)(1-\delta_c)$

$$\boxed{c_{ij}^* = 0}$$

**Conductance** : $c_{ij}^* = 0 < c_\theta$ → affaiblissement systématique :

$$w_{ij}(t+1) = w_{ij}(t) - \eta_- \cdot p_{ij}$$

La conductance décroît linéairement jusqu'au plancher :

$$\boxed{w_{ij}^* = w_{\min}}$$

#### État de repos complet

$$\boxed{\mathbf{x}^*_{\text{repos}} = \left(a_{\min},\; 0,\; 0,\; 0,\; 1,\; w_{\min},\; 0\right)}$$

### 9.3 Stabilité de l'équilibre de repos

L'état de repos est stable si et seulement si **aucun nœud n'est actif à l'état de repos** :

$$a_{\min} \leq \theta_i^{\text{eff}} = \frac{\theta_i + 0 + 0}{1} = \theta_i$$

$$\boxed{a_{\min} < \theta_i \quad \forall i}$$

Avec les valeurs par défaut ($a_{\min} = 0{,}01$, $\theta = 0{,}2$) : **la condition est largement satisfaite**.

Si cette condition était violée ($a_{\min} \geq \theta$), tous les nœuds seraient actifs en permanence et le système ne pourrait jamais atteindre le repos → **saturation globale**.

### 9.4 Vitesse de convergence vers le repos

La constante de temps dominante est celle de la trace mémoire (decay le plus lent).

Pour chaque variable en régime libre :

| Variable | Récurrence | Demi-vie ($\tau_{1/2}$) |
|---|---|---|
| $a_i$ | $\times(1-\alpha)$ | $\frac{\ln 2}{\ln(1/(1-\alpha))}$ |
| $f_i$ | $\times(1-\rho_f)$ | $\frac{\ln 2}{\ln(1/(1-\rho_f))}$ |
| $h_i$ | $\times(1-\delta_h)$ | $\frac{\ln 2}{\ln(1/(1-\delta_h))}$ |
| $m_i$ | $\times(1-\delta_m)$ | $\frac{\ln 2}{\ln(1/(1-\delta_m))}$ |
| $c_{ij}$ | $\times(1-\delta_c)$ | $\frac{\ln 2}{\ln(1/(1-\delta_c))}$ |
| $w_{ij}$ | $-\eta_- p_{ij}$/tick | $\frac{w_0 - w_{\min}}{2\eta_- p_{ij}}$ |

Avec les paramètres par défaut :

| Variable | Taux | Demi-vie (ticks) |
|---|---|---|
| Activation | $\alpha = 0{,}25$ | $\approx 2{,}4$ |
| Fatigue | $\rho_f = 0{,}05$ | $\approx 13{,}5$ |
| Inhibition | $\delta_h = 0{,}03$ | $\approx 22{,}8$ |
| Trace mémoire | $\delta_m = 0{,}005$ | $\approx 138{,}3$ |
| Co-activité | $\delta_c = 0{,}05$ | $\approx 13{,}5$ |
| Conductance | $\eta_- = 0{,}002$ | $\approx 250$ ticks (linéaire) |

**La trace mémoire et la conductance définissent la mémoire longue du système** (~140 et ~250 ticks respectivement).

---

## 10. Recherche d'un équilibre actif

### 10.1 Existence

Un **équilibre actif** est un état stationnaire avec au moins un nœud actif : $\exists i : \sigma_i^* = 1$.

Pour qu'un nœud $i$ reste actif indéfiniment, il faut que l'énergie reçue par propagation compense exactement les pertes par decay :

$$a_i^* = \frac{a_i^* + \varphi_i^*}{1} \cdot (1 - \alpha) \quad \Longleftrightarrow \quad a_i^* = a_i^*(1-\alpha) + \varphi_i^*(1-\alpha)$$

$$\boxed{a_i^* = \frac{\varphi_i^*(1-\alpha)}{\alpha}}$$

Pour que ce nœud soit actif :

$$a_i^* > \theta_i^{\text{eff}*}$$

### 10.2 Condition nécessaire pour un équilibre actif auto-entretenu

Considérons un ensemble $S \subseteq V$ de nœuds actifs à l'équilibre. Pour chaque $i \in S$ :

$$a_i^* = \frac{(1-\alpha)}{\alpha} \sum_{j \in S \cap \mathcal{N}^-(i)} a_j^* \cdot w_{ji}^* \cdot K(d_{ji}) \cdot g$$

C'est un système linéaire en $\mathbf{a}_S^*$. En notant $M$ la matrice :

$$M_{ik} = \frac{(1-\alpha)}{\alpha} \cdot w_{ki}^* \cdot K(d_{ki}) \cdot g \quad \text{pour } k \in S \cap \mathcal{N}^-(i)$$

L'existence d'un point fixe non trivial requiert :

$$\boxed{\rho(M) = 1}$$

où $\rho(M)$ est le **rayon spectral** de $M$ (plus grande valeur propre en module).

- Si $\rho(M) < 1$ : toute activité s'éteint (repos stable)
- Si $\rho(M) > 1$ : l'activité explose (saturation)
- Si $\rho(M) = 1$ : équilibre actif marginal (instable en pratique)

### 10.3 Complication : les boucles de rétroaction

En réalité, un équilibre actif auto-entretenu pur est **structurellement instable** dans ce système, à cause des rétroactions négatives :

**La fatigue croît avec l'activité** : si $\sigma_i = 1$ en permanence :

$$f_i(t+1) = (f_i(t) + \gamma_f a_i)(1-\rho_f) \xrightarrow{t \to \infty} f_i^{*} = \frac{\gamma_f a_i^*}{\rho_f}$$

**L'inhibition croît avec l'activité** :

$$h_i^* = \frac{\gamma_h}{\delta_h}$$

Donc le seuil effectif à l'équilibre actif serait :

$$\theta_i^{\text{eff}*} = \frac{\theta_i + \frac{\gamma_f a_i^*}{\rho_f} + \frac{\gamma_h}{\delta_h}}{\varepsilon_i^*}$$

Avec les valeurs par défaut :

$$\frac{\gamma_h}{\delta_h} = \frac{0{,}12}{0{,}03} = 4{,}0$$

$$\theta_i^{\text{eff}*} \geq \frac{0{,}2 + 0 + 4{,}0}{1{,}5} \approx 2{,}8$$

Le seuil effectif monterait à **~2,8** minimum, requérant une activation auto-entretenue de $a_i > 2{,}8$ pour chaque nœud actif. C'est **extrêmement coûteux** en termes d'influence entrante.

### 10.4 Conclusion sur l'équilibre actif

Le système est **conçu pour ne pas avoir d'équilibre actif stable**.

Les rétroactions négatives (fatigue + inhibition) augmentent progressivement le seuil effectif de tout nœud actif, finissant par éteindre l'activité. Ceci est un **comportement voulu** : le système produit des **ondes transitoires**, pas des oscillations entretenues.

Le seul état stationnaire est le **repos** :

$$\boxed{\mathbf{x}^* = \left(a_{\min},\; 0,\; 0,\; 0,\; 1,\; w_{\min},\; 0\right)}$$

---

## 11. Régimes dynamiques transitoires

Bien que le système n'ait pas d'équilibre actif, il présente des **régimes transitoires caractéristiques** :

### 11.1 Phase de propagation (onde)

Après un stimulus ponctuel d'intensité $I$ au nœud $i_0$ :

- **Front d'onde** : expansion spatiale tant que l'influence reçue dépasse le seuil effectif des voisins.
- **Rayon maximal** approximé par la condition : $I \cdot \prod_{k=1}^{r} \bar{w} \cdot K(\bar{d}) \cdot g \cdot (1-\alpha)^r > \theta$

En simplifiant avec un facteur d'amplification moyen par saut $\mu$ :

$$\mu = \bar{w} \cdot K(\bar{d}) \cdot g \cdot \frac{(1-\alpha)}{\alpha} \cdot \bar{k}$$

où $\bar{k}$ est le nombre moyen de voisins. Le rayon maximal de l'onde est :

$$r_{\max} \approx \frac{\ln\left(\theta / I\right)}{\ln(\mu)} \quad \text{(si } \mu > 1\text{)}$$

### 11.2 Phase de dissipation

Après le passage de l'onde, sans réinjection, la relaxation suit :

$$a_i(t + \Delta t) \approx a_i(t_{\text{pic}}) \cdot (1-\alpha)^{\Delta t} + a_{\min}\left(1 - (1-\alpha)^{\Delta t}\right)$$

Temps pour atteindre $a_{\min} + \epsilon$ depuis un pic $a_{\text{pic}}$ :

$$\Delta t_{\text{relax}} \approx \frac{\ln((a_{\text{pic}} - a_{\min}) / \epsilon)}{\ln(1/(1-\alpha))}$$

### 11.3 Phase de mémoire résiduelle

Après l'onde, les traces persistent :

- $m_i$ décroît lentement ($\tau_{1/2} \approx 138$ ticks)
- $w_{ij}$ renforcées décroissent vers $w_{\min}$ ($\tau \approx 250$ ticks)
- L'excitabilité $\varepsilon_i$ reste élevée tant que $m_i > 0$

Cela crée une **fenêtre de facilitation** : un second stimulus identique dans cette fenêtre produit une onde plus large (seuil effectif plus bas via $\varepsilon > 1$) et plus rapide (conductances plus élevées).

---

## 12. Condition de non-saturation

La contrainte critique pour éviter l'explosion globale :

Le facteur d'amplification par saut doit satisfaire :

$$\boxed{\mu = \bar{k} \cdot \bar{w} \cdot K(\bar{d}) \cdot g \cdot \frac{1-\alpha}{\alpha} < \mu_{\text{crit}}}$$

où $\mu_{\text{crit}}$ dépend de la topologie. Empiriquement :

- $\mu_{\text{crit}} \approx 1{,}0$ pour un graphe homogène (tous les nœuds conduisent à la saturation)
- $\mu_{\text{crit}} \approx 1{,}5 - 2{,}0$ pour les KNN 3D (variabilité des distances et degrés créant une atténuation naturelle)

Avec les paramètres calibrés V1 ($\bar{k} = 10$, $\bar{w} = 1$, $\lambda = 0{,}15$, $\bar{d} \approx 18$ pour un cube 100³ avec 10k nœuds, $g = 0{,}6$, $\alpha = 0{,}25$) :

$$K(\bar{d}) = e^{-0{,}15 \times 18} \approx 0{,}067$$

$$\mu \approx 10 \times 1 \times 0{,}067 \times 0{,}6 \times \frac{0{,}75}{0{,}25} = 10 \times 0{,}040 \times 3 = 1{,}21$$

Le système est dans la zone $\mu > 1$ : **l'onde se propage** mais les rétroactions (fatigue, inhibition) la freinent avant saturation totale.

---

## 13. Énergie globale

L'**énergie globale** du système est définie comme :

$$E(t) = \sum_{i=1}^{N} a_i(t)$$

À l'équilibre de repos :

$$\boxed{E^* = N \cdot a_{\min}}$$

Le ratio $E(t) / E^*$ mesure le **surcoût énergétique** lié à l'activité transitoire.

---

## 14. Tableau récapitulatif des équations

| # | Équation | Phase |
|---|---|---|
| E1 | $a_i \leftarrow a_i + I_s$ | Stimulus |
| E2 | $\varphi_i = \sum_{j} \sigma_j \cdot a_j \cdot w_{ji} \cdot K(d_{ji}) \cdot g$ | Propagation |
| E3 | $a_i \leftarrow \min(a_i + \varphi_i,\; 10)$ | Propagation |
| E4 | $f_i \leftarrow (f_i + \gamma_f a_i \sigma_i)(1-\rho_f)$ | Dissipation |
| E5 | $h_i \leftarrow (h_i + \gamma_h \sigma_i)(1-\delta_h)$ | Dissipation |
| E6 | $m_i \leftarrow (m_i + \gamma_m a_i \sigma_i)(1-\delta_m)$ | Dissipation |
| E7 | $\varepsilon_i \leftarrow 1 + 0{,}1 \cdot \min(m_i, 5)$ | Dissipation |
| E8 | $a_i \leftarrow \max(a_i(1 - \alpha_i^{\text{eff}}),\; a_{\min})$ | Dissipation |
| E9 | $c_{ij} \leftarrow [\text{co-act}](1-\delta_c)$ | Plasticité |
| E10 | $w_{ij} \leftarrow \text{clamp}(w_{ij} + \Delta w_{ij})$ | Plasticité |

---

## 15. Résumé des points d'équilibre

| Régime | Condition | État |
|---|---|---|
| **Repos** (unique stable) | $a_{\min} < \theta$ | $a^* = a_{\min}$, $f^*=h^*=m^*=c^*=0$, $\varepsilon^*=1$, $w^* = w_{\min}$ |
| **Saturation** (instable) | $\mu \gg 1$ et fatigue/inhibition insuffisants | tous les nœuds saturés à $a = 10$ |
| **Onde transitoire** (dynamique, pas un point fixe) | $\mu > 1$ avec rétroactions suffisantes | propagation + dissipation → retour au repos |
