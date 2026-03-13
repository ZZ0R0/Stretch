# Modélisation mathématique — V4

## 1. Objet

Définir un formalisme mathématique minimal pour la V4.

La V4 ajoute :
- reward externe ;
- dopamine minimale ;
- trace d’éligibilité ;
- modulation de la plasticité ;
- entrées/sorties minimales.

---

## 2. Reward externe

On définit un signal de récompense :

```text
r(t) ∈ [-1, +1]
```

Il peut être :
- exogène ;
- dérivé d’une sortie correcte/incorrecte.

Reward cumulé :

```text
R_cum(T) = Σ_{t=0..T} r(t)
```

---

## 3. Signal dopaminergique

Version minimale :

```text
d(t+1) = (1 - lambda_d) * d(t) + g_r * r(t)
```

où :
- `d(t)` = niveau dopaminergique effectif ;
- `lambda_d` = decay ;
- `g_r` = gain reward → dopamine.

Version tonique + phasique :

```text
d_total(t) = d_tonic + d_phasic(t)
d_phasic(t+1) = (1 - lambda_dp) * d_phasic(t) + g_r * r(t)
```

---

## 4. Trace d’éligibilité

Pour chaque arête `(i → j)` :

```text
e_ij(t+1) = gamma_e * e_ij(t) + psi_ij(t)
```

où :
- `gamma_e ∈ [0,1]` = decay ;
- `psi_ij(t)` = événement local de plasticité récente.

Version simple :

```text
psi_ij(t) = Delta_w_STDP_local(t)
```

Version plus stable :

```text
psi_ij(t) = alpha_pre * pre_i(t) + alpha_post * post_j(t) + alpha_stdp * Delta_w_STDP_local(t)
```

---

## 5. STDP asymétrique

Pour une différence temporelle :

```text
Delta_t = t_post - t_pre
```

on définit :

```text
si Delta_t > 0 :
    Delta_w_STDP = +A_plus * exp(-Delta_t / tau_plus)

si Delta_t < 0 :
    Delta_w_STDP = -A_minus * exp(Delta_t / tau_minus)
```

avec contrainte V4 recommandée :

```text
A_plus > A_minus
```

---

## 6. Plasticité modulée par reward

Poids effectif mis à jour par :

```text
Delta_w_eff(t) = Delta_w_STDP(t) + eta_r * d(t) * e_ij(t)
```

ou, version séparée :

```text
w_ij(t+1) = clamp(
    w_ij(t)
    + Delta_w_STDP(t)
    + eta_rew * d(t) * e_ij(t),
    w_min,
    w_max
)
```

Cette forme permet :
- STDP seule si `eta_rew = 0` ;
- reward-modulated STDP si `eta_rew > 0`.

---

## 7. Gating de consolidation

La consolidation ne doit s’appliquer que si :

```text
gate_consol_ij(t) = 1
```

et par exemple :

```text
gate_consol_ij(t) = 1
si [ d(t) > d_thresh ] ET [ e_ij(t) > e_thresh ]
sinon 0
```

Version plus stricte :

```text
gate_consol_ij(t) = 1
si [ a_ij_event(t) ] ET [ d(t) > d_thresh ] ET [ w_ij > w_consol_eff ]
```

---

## 8. Entrée minimale

Pattern externe `x(t)` encodé sur une zone d’entrée `Z_in`.

Pour `k` neurones d’entrée :

```text
a_i(t) += Enc_i(x(t))   pour i ∈ Z_in
```

où `Enc_i` peut être :
- binaire ;
- sparse ;
- intensité simple.

---

## 9. Sortie minimale

On définit des groupes de sortie `G_1, ..., G_n`.

Score de sortie :

```text
score_m(t) = Σ_{i ∈ G_m} a_i(t)
```

Décision :

```text
y_hat(t) = argmax_m score_m(t)
```

Accuracy binaire ou multi-classe :

```text
acc = mean( y_hat == y_true )
```

Reward simple supervisé :

```text
r(t) = +1 si y_hat = y_true
r(t) = -1 sinon
```

---

## 10. Condition pratique de succès V4

La V4 est considérée comme mathématiquement prometteuse si l’on observe simultanément :

- `d(t)` module la plasticité sans déstabiliser l’activation ;
- `e_ij(t)` conserve l’information locale sur quelques ticks ;
- `r(t)` différencie les poids appris ;
- l’entrée encode des patterns distincts ;
- la sortie permet une décision meilleure que le hasard.
