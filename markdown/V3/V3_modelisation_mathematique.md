# Modélisation mathématique — V3

## 1. Objet

Définir un formalisme mathématique minimal pour la V3.

La V3 ajoute :
- propagation signée excitatrice / inhibitrice ;
- contrôle indirect ;
- plasticité STDP ;
- sélectivité mémoire minimale.

---

## 2. Types neuronaux

Pour chaque nœud i :

- type(i) ∈ {E, I, C, P}

où :
- E = excitateur
- I = inhibiteur
- C = contrôle
- P = pacemaker

---

## 3. Propagation signée

Influence totale reçue par le nœud i :

```text
phi_i(t) = Σ_j sigma_j(t) * a_j(t) * s_j * w_ji(t) * K(d_ji) * g_zone(j)
```

où :

- `sigma_j(t)` = prédicat d’activité
- `s_j = +1` si j est excitateur
- `s_j = -beta_I` si j est inhibiteur
- `K(d)` = noyau spatial
- `g_zone(j)` = gain régional modulé par le PID indirect

Exemple minimal :
```text
K(d) = exp(-lambda * d)
```

---

## 4. Seuil effectif avec modulation régionale

```text
theta_i_eff(t) =
(theta_i + f_i(t) + h_i(t) + theta_zone_mod(zone(i), t))
/
max(eps_i(t) + eps_zone_mod(zone(i), t), eps_min)
```

Le PID n’ajoute donc pas directement à `a_i`, mais modifie :
- le seuil effectif ;
- le gain régional ;
- éventuellement l’excitabilité effective.

---

## 5. PID indirect

Pour une zone Z :

```text
a_Z(t) = (1 / |Z|) * Σ_{i in Z} a_i(t)
e_Z(t) = a_target_Z - a_Z(t)
I_Z(t+1) = I_Z(t) + e_Z(t) * dt
D_Z(t) = (e_Z(t) - e_Z(t-1)) / dt
u_Z(t) = Kp * e_Z(t) + Ki * I_Z(t) + Kd * D_Z(t)
```

Projection possible de u_Z :

```text
theta_zone_mod = -k_theta * u_Z
g_zone_mod     = +k_g * u_Z
eps_zone_mod   = +k_eps * u_Z
```

Les coefficients de projection doivent être bornés.

---

## 6. STDP

Chaque arête (i → j) conserve :
- `last_pre_tick`
- `last_post_tick`

On définit :

```text
Delta_t = t_post - t_pre
```

Mise à jour STDP minimale :

```text
si Delta_t > 0 :
    Delta_w = +A_plus * exp(-Delta_t / tau_plus)

si Delta_t < 0 :
    Delta_w = -A_minus * exp(Delta_t / tau_minus)
```

Donc :

```text
w_ij(t+1) = clamp(w_ij(t) + Delta_w, w_min, w_max)
```

---

## 7. Sélectivité mémoire minimale

Exemple 1 — budget synaptique sortant :

```text
Σ_j w_ij <= W_budget
```

Si dépassé :
- renormalisation ;
- ou punition des arêtes les moins performantes.

Exemple 2 — seuil adaptatif de consolidation :

```text
w_consol_eff(i) = w_consol_base + alpha * mean_outgoing_weight(i)
```

Exemple 3 — gating événementiel :

la consolidation n’est autorisée que si :

```text
a_i(t) > a_event_threshold
```

ou si l’écart au fond d’activité de zone dépasse un seuil.

---

## 8. Oscillations émergentes

La V3 ne suppose pas une solution analytique complète, mais vise l’apparition de régimes où :

```text
E(t) et I(t)
```

forment un couple dynamique oscillant.

Mesures recommandées :
- fréquence dominante FFT ;
- phase relative E/I ;
- stabilité sur fenêtre ;
- amplitude.

---

## 9. Condition pratique de succès V3

La V3 est considérée comme mathématiquement prometteuse si l’on observe simultanément :

- propagation auto-entretenue sans injection directe PID ;
- contraste spatial dû à l’inhibition ;
- apprentissage directionnel STDP ;
- oscillation locale émergente ;
- sélectivité mémoire croissante.
