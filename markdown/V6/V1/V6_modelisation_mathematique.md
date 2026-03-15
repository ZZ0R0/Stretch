# V6 — Modélisation Mathématique

## 1. Compétition de sparsité à front d'onde

### Score de compétition

Pour chaque neurone $i$ au tick $t$ :

$$s_i(t) = \phi_i(t) \times \beta_i(t)$$

où $\beta_i(t)$ est le **bonus de nouveauté** :

$$\beta_i(t) = 1 + \lambda_{\text{nov}} \times \frac{\max\bigl(0,\; W - (t - t_i^{\text{first}})\bigr)}{W}$$

- $t_i^{\text{first}}$ = tick de première activation dans le trial courant ($+\infty$ si jamais activé)
- $W$ = fenêtre de nouveauté (ticks)
- $\lambda_{\text{nov}}$ = gain de nouveauté

**Comportement** :
- Neurone fraîchement activé ($t - t_i^{\text{first}} = 0$) : bonus = $1 + \lambda_{\text{nov}}$
- Neurone activé il y a $W$ ticks : bonus = 1 (pas de bonus)
- Neurone activé il y a $> W$ ticks : bonus = 1 (pas de pénalité)

### Sélection top-K

Soit $K = \lfloor f_{\max} \times N \rfloor$ le budget de sparsité ($N$ = nombre total de neurones).

On sélectionne les $K$ neurones de plus grand score $s_i(t)$. 

Pour les neurones $j$ **non sélectionnés** :

$$\phi_j(t) \leftarrow \phi_j(t) \times f_{\text{suppress}}$$

avec $f_{\text{suppress}} \in [0, 1]$ le facteur de suppression.

### Propriétés du front d'onde

**Théorème informel** : Avec $\lambda_{\text{nov}} > 0$ et $W > 0$, le front d'onde
avance d'au moins 1 hop toutes les $W$ ticks sous sparsité.

*Argument* : Les neurones du front (nouvellement activés) ont un score majoré par
$(1 + \lambda_{\text{nov}})$ par rapport aux neurones du corps (activés depuis $> W$ ticks).
Donc même si $\phi_{\text{corps}} > \phi_{\text{front}}$, tant que :

$$\phi_{\text{front}} \times (1 + \lambda_{\text{nov}}) > \phi_{\text{corps}}$$

le front gagne la compétition. Avec $\lambda_{\text{nov}} = 2$, le front survit tant que
$\phi_{\text{front}} > \phi_{\text{corps}} / 3$, ce qui est largement vérifié pour un
front à 1-2 hops du corps.

## 2. Modulation dopaminergique

### Sigmoïde de modulation

$$\sigma_d = \frac{1}{1 + \exp\left(\frac{d(t) - \theta_d}{\kappa}\right)}$$

- $d(t)$ = niveau de dopamine au tick $t$
- $\theta_d$ = seuil de dopamine (typique : tonic = 0.1)
- $\kappa$ = pente ($> 0$, typique : 0.05)

$\sigma_d$ est proche de 1 quand $d < \theta$ (recherche), proche de 0 quand $d > \theta$ (exploitation).

### Réverbération modulée (EF-6)

$$r_{\text{eff}}(t) = r_{\min} + (r_{\max} - r_{\min}) \times \sigma_d(t)$$

| Phase | $d(t)$ | $\sigma_d$ | $r_{\text{eff}}$ |
|-------|--------|-----------|------------------|
| Recherche | 0.1 | ~1.0 | 0.30 |
| Exploitation | 0.5 | ~0.0 | 0.05 |

### Decay modulé (EF-7)

$$\alpha_{\text{eff}}(t) = \alpha_{\text{base}} \times \bigl(1 - \mu_{\text{decay}} \times \sigma_d(t)\bigr)$$

| Phase | $d(t)$ | $\sigma_d$ | $\alpha_{\text{eff}}$ (base=0.25) |
|-------|--------|-----------|----------------------------------|
| Recherche | 0.1 | ~1.0 | 0.175 (−30%) |
| Exploitation | 0.5 | ~0.0 | 0.25 (inchangé) |

## 3. Stabilité énergétique sous sparsité

### Énergie totale

$$E(t) = \sum_{i=1}^{N} \phi_i(t) \leq K \times \phi_{\max} = f_{\max} \times N \times 10$$

Avec $f_{\max} = 0.05$ et $N = 50\,000$ : $E_{\max} = 25\,000$

En pratique, les activations typiques sont ~0.5–2.0, donc :
$$E_{\text{typique}} \approx 2500 \times 1.0 = 2\,500$$

Ce qui est ~10× inférieur au cas non-sparse (~25 000 neurones actifs × 1.0).

### G_eff sous sparsité

Le gain effectif par hop change sous sparsité :

$$G_{\text{eff}} = G \times \langle k \rangle \times f_{\text{active}} \times \langle C \rangle$$

Avec $G = 0.8$, $\langle k \rangle = 10$, $f_{\text{active}} = 0.05$, $\langle C \rangle = 1.0$ :

$$G_{\text{eff}} = 0.8 \times 10 \times 0.05 \times 1.0 = 0.4 < 1$$

Le réseau est **stable sous sparsité** (pas de divergence).

## 4. Vitesse de propagation

Sans sparsité, le front d'onde avance de ~1 hop/tick.

Avec sparsité + wavefront bonus : le mécanisme de sélection favorise les neurones
de la frontière, donc la vitesse reste ~1 hop/tick.

Le temps de traversée $T_{\text{cross}}$ pour une distance $D$ en hops :

$$T_{\text{cross}} \approx D \text{ ticks}$$

Pour un réseau 3D de côté 100 avec $k=10$ :
- Distance moyenne input→output (Symmetric) : ~60 unités ≈ 15-20 hops
- Temps de traversée : ~15-20 ticks
- Trial = 5 ticks stimulus + 20 ticks delay → signal devrait atteindre l'output
