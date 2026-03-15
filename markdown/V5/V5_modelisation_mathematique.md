# Modélisation mathématique — V5

## 1. Objet

Définir un formalisme mathématique minimal pour la V5.

La V5 ajoute :
- calibration multi-échelle ;
- mécanismes de dynamique soutenue ;
- métriques de preuve d’apprentissage ;
- chemins conductifs.

---

## 2. Baselines

### 2.1 Topology-only baseline
```text
Delta_w = 0
```
ou
```text
plasticity disabled
```

### 2.2 Full learning
```text
w_ij(t+1) = w_ij(t) + Delta_w_STDP + eta_rew * d(t) * e_ij(t) + Delta_homeostasis
```

---

## 3. Calibration multi-échelle

La V5 recommande des règles adaptatives du type :

```text
group_size(n) = max(group_size_min, sqrt(n))
```

```text
gain(n) = g_ref * log(n) / log(n_ref)
```

```text
eligibility_decay(n) ∈ [0.95, 0.98]
```

```text
edge_decay(n) = edge_decay_ref / (n / n_ref)
```

```text
target_activity(n) = min(target_ref, K / n)
```

Ces lois ne sont pas imposées comme vérité finale, mais comme hypothèses de calibration testables.

---

## 4. Dynamique soutenue

### 4.1 Decay adaptatif
Au lieu de :

```text
a_i(t+1) = a_i(t) * (1 - alpha)
```

on peut utiliser :

```text
a_i(t+1) = a_i(t) * (1 - alpha_eff_i(t))
```

avec :

```text
alpha_eff_i(t) = alpha_base * (1 - k_local * local_activity_i(t))
```

borné dans un intervalle stable.

### 4.2 Réverbération locale
Sur une zone locale :

```text
phi_i(t) = Σ_j sigma_j(t) * a_j(t) * s_j * w_ji * K(d_ji) * g
```

on peut ajouter :

```text
phi_i_eff(t) = phi_i(t) + r_local * phi_i(t-1)
```

ou une boucle courte de feedback spatialement bornée.

---

## 5. Conductance directionnelle

Pour un input `I_k` et une sortie `O_m`, on définit un score de route :

```text
RouteScore(I_k, O_m) = Σ_{(u→v) in P*} w_uv
```

où `P*` est le plus court chemin ou le chemin de conductance maximale selon le critère choisi.

Un indice directionnel minimal :

```text
D_k = RouteScore(I_k, O_target) - RouteScore(I_k, O_competitor)
```

La V5 doit montrer :

```text
D_k > 0
```

sur une tâche apprise pertinente.

---

## 6. Cohérence topologique du renforcement

On définit un indicateur minimal :

```text
CT = corr(Delta_w_ij, OnUsefulPath_ij)
```

où :
- `Delta_w_ij` = variation de conductance ;
- `OnUsefulPath_ij` = 1 si l’arête appartient à une route utile, sinon 0.

Une V5 réussie doit montrer un `CT` significativement positif sur au moins une tâche.

---

## 7. Mesure de la dynamique soutenue

Exemple de ratio :

```text
SustainRatio = mean_energy_intertrial / mean_energy_peak
```

En V4 ce ratio est très faible.  
La V5 doit viser une augmentation de ce ratio sans saturation globale.

---

## 8. Condition pratique de succès V5

La V5 est considérée comme mathématiquement prometteuse si :

- au moins une tâche anti-biais est apprise ;
- `CT > 0` de manière reproductible ;
- `D_k` devient favorable à la sortie cible ;
- `SustainRatio` augmente significativement ;
- la performance reste au-dessus du hasard sur plusieurs échelles de `n`.
