# Aboutissements V2

## Résumé

La V2 accomplit la transformation fondamentale d'un **réseau purement réactif** (V1 : stimulus → propagation → extinction) en un **système auto-régulé** capable de maintenir une activité dynamique stable indéfiniment. C'est la version critique de toute la roadmap : sans elle, aucune cognition ne serait envisageable.

Trois mécanismes majeurs ont été introduits : le **partitionnement spatial en zones avec régulateurs PID**, les **oscillateurs pacemaker**, et la **consolidation mémoire structurelle**. Le système fonctionne à 50 000 nœuds, 577 000 arêtes, en mode infini (~21 ms/tick sur 1 CPU).

---

## 1. Régulation PID par zones — Activité endogène

### Réalisation

C'est le cœur de la V2. Chaque zone spatiale est gouvernée par un contrôleur PID qui mesure l'activité moyenne locale et injecte une correction pour la maintenir à une consigne $a_{\text{target}}$.

#### Partitionnement de Voronoï

- L'espace 3D est partitionné en $K$ zones par Voronoï : $K$ centres sont choisis parmi les nœuds (échantillonnage régulier pour reproductibilité), et chaque nœud est assigné au centre le plus proche.
- Le centre de chaque zone est le **neurone de contrôle** (NC) : il ne fait pas partie des membres régulés.
- Implémenté dans `stretch-core/src/zone.rs`.

#### Contrôleur PID

Pour chaque zone $Z_k$ à chaque tick :

1. **Mesure** : $\bar{a}_k(t) = \frac{1}{|Z_k|} \sum_{i \in Z_k} a_i(t)$
2. **Erreur** : $e_k(t) = a_{\text{target}} - \bar{a}_k(t)$
3. **Intégrale** (avec anti-windup) : $I_k(t) = \text{clamp}(I_k(t-1) + e_k(t),\; -I_{\max},\; I_{\max})$
4. **Dérivée** : $D_k(t) = e_k(t) - e_k(t-1)$
5. **Sortie** : $u_k(t) = \text{clamp}(K_p \cdot e_k + K_i \cdot I_k + K_d \cdot D_k,\; -u_{\max},\; u_{\max})$
6. **Injection** : chaque nœud membre reçoit $u_k$ directement :
   $a_i(t) \leftarrow \text{clamp}(a_i(t) + u_k,\; 0,\; 10)$

#### Paramètres calibrés

| Paramètre | Symbole | Valeur (zones) | Valeur (training) |
|---|---|---|---|
| Zones | $K$ | 8 | 12 |
| Consigne | $a_{\text{target}}$ | 0.3 | 0.25 |
| Gain P | $K_p$ | 0.5 | 0.4 |
| Gain I | $K_i$ | 0.05 | 0.03 |
| Gain D | $K_d$ | 0.1 | 0.08 |
| Borne sortie | $u_{\max}$ | 2.0 | 1.5 |
| Borne intégrale | $I_{\max}$ | 5.0 | 4.0 |

### Résultat observé

- **Activité endogène stable** : ~9 000 nœuds actifs en permanence sur 50 000 (18%), avec énergie oscillant entre ~109 000 et ~115 000.
- **Pas d'extinction** : le PID maintient l'activité indéfiniment, même sans aucun stimulus externe.
- **Erreur PID moyenne** converge vers ~0,004 (quasi-parfait) après ~100 ticks de transition.
- **Pas de divergence** : les bornes anti-windup ($u_{\max}$, $I_{\max}$) empêchent l'emballement intégral.

### Instabilité V1 résolue

- ✅ **I2 — Pas d'équilibre actif** : résolu. Le PID crée un équilibre actif stable à $\bar{a} \approx a_{\text{target}}$.
- ✅ **I4 — Pas d'activité endogène** : résolu. Le système génère et maintient sa propre activité.
- ✅ **I7 — Pas de régulation locale** : résolu. Chaque zone est régulée indépendamment.

---

## 2. Oscillateurs pacemaker

### Réalisation

Des nœuds spécifiques sont désignés comme **pacemakers** et reçoivent une injection sinusoïdale intrinsèque à chaque tick :

$$a_i(t) \leftarrow \text{clamp}(a_i(t) + A \sin(2\pi f t + \varphi) + o,\; 0,\; 10)$$

- Chaque pacemaker possède ses propres paramètres : amplitude $A$, fréquence $f$, phase $\varphi$, offset $o$.
- L'injection est signée : les valeurs négatives produisent une inhibition légère.
- Implémenté dans `stretch-core/src/pacemaker.rs`.

### Configurations testées

Config pacemakers (3 oscillateurs) :

| Pacemaker | Nœud | $A$ | $f$ (cycles/tick) | $\varphi$ | $o$ |
|---|---|---|---|---|---|
| PM1 | 100 | 0.30 | 0.020 | 0 | 0.5 |
| PM2 | 3 000 | 0.25 | 0.030 | 0 | 0.5 |
| PM3 | 7 000 | 0.20 | 0.015 | $\pi$ | 0.4 |

Config training (2 oscillateurs) :

| Pacemaker | Nœud | $A$ | $f$ | $\varphi$ | $o$ |
|---|---|---|---|---|---|
| PM1 | 500 | 0.30 | 0.020 | 0 | 0.4 |
| PM2 | 5 000 | 0.25 | 0.025 | $\pi/2$ | 0.4 |

### Résultat observé

- Les oscillations se propagent localement autour du pacemaker.
- Avec des fréquences différentes, on observe des **battements** : les zones de pacemaker voisins voient des modulations d'amplitude.
- Le PID interagit avec les pacemakers : quand l'oscillation positive pousse l'activité au-dessus de la consigne, le PID réduit son injection, et vice-versa. Cela crée une dynamique riche.

### Instabilité V1 résolue

- ✅ **I6 — Pas d'oscillations** : résolu. Les pacemakers produisent des oscillations sinusoïdales persistantes.

---

## 3. Consolidation mémoire structurelle

### Réalisation

Les arêtes dont la conductance reste au-dessus d'un seuil pendant suffisamment de ticks deviennent **consolidées** : leur decay est désactivé, et seul le renforcement reste possible.

Mécanisme par arête :

```
si w_ij >= threshold pendant ticks_required ticks consécutifs :
    consolidated = true
    → plus d'affaiblissement (weakening = 0)
    → le decay ne peut que renforcer
si w_ij < threshold à un moment :
    compteur remis à 0
```

Implémenté dans `stretch-core/src/edge.rs` : champs `consolidation_counter` et `consolidated`.

### Paramètres

| Paramètre | Symbole | Valeur (zones) | Valeur (training) |
|---|---|---|---|
| Seuil | $w_{\text{consol}}$ | 2.5 | 2.0 |
| Durée requise | $T_{\text{consol}}$ | 50 ticks | 40 ticks |

### Résultat observé

- La consolidation fonctionne : les arêtes fortement utilisées deviennent permanentes.
- **Problème identifié** : avec le PID actif, l'activité homogène fait que toutes les arêtes voient de la co-activation. Avec un seuil de 2.0–2.5, **la quasi-totalité des arêtes se consolident** (~575 000 / 577 000). La consolidation n'est pas sélective.

### Instabilité V1 résolue

- ✅ **I3 — Conductance → $w_{\min}$** : résolu. Les arêtes consolidées ne subissent plus de decay.
- ⚠️ Mais la sélectivité est insuffisante (voir section instabilités V2).

---

## 4. Cycle de simulation V2 — 6 phases

### Séquence d'un tick

```
Phase 0 : Mesure        — chaque zone calcule ā_zone
Phase 1 : Régulation    — PID calcule u et l'injecte
Phase 2 : Stimulus      — injections externes (si actifs au tick courant)
Phase 2b: Pacemakers    — oscillations sinusoïdales
Phase 3 : Propagation   — noyaux exponentiels/gaussiens (hérité V1)
Phase 4 : Dissipation   — fatigue, inhibition, trace, decay avec jitter
Phase 5 : Plasticité    — Hebbian + consolidation
Phase 6 : Métriques     — snapshot
```

L'ordre est important : la régulation PID agit **avant** la propagation, permettant de préparer le terrain. Les pacemakers injectent **après** les stimuli externes mais **avant** la propagation, pour que leur oscillation soit propagée.

### Mode infini

- `total_ticks = 0` signifie simulation infinie (la simulation ne s'arrête que par signal utilisateur : Q, ESC, ou timeout).
- L'affichage dans la barre de titre montre uniquement le tick courant sans borne.

---

## 5. Performances et scalabilité

### Benchmarks observés

| Configuration | Nœuds | Arêtes | Ticks/s (CLI) | ms/tick |
|---|---|---|---|---|
| `config_v2_zones` | 50 000 | 577 488 | ~48 | ~21 |
| `config_v2_training` | 50 000 | 577 488 | ~48 | ~21 |
| `config_v2_pacemakers` | 50 000 | 577 488 | ~48 | ~21 |

- La viz tourne à ~45 FPS en mode 1 tick/frame.
- Le surcoût V2 (mesure zones + PID + pacemakers + consolidation) est négligeable par rapport à la propagation (~577k arêtes à parcourir).
- Le scaling reste linéaire en $O(N \times k)$ où $k$ est le nombre moyen de voisins.

### Rétro-compatibilité V1

- Toutes les extensions V2 sont optionnelles : `zones.enabled = false`, `pacemakers = []`, `consolidation.enabled = false`.
- Les configs V1 fonctionnent sans modification.
- Validé avec `config_v1_knn.toml` : résultats identiques à V1.

---

## 6. Visualisation V2

### Améliorations

- **Panneau ZONES PID** dans la sidebar : nombre de zones, activité moyenne, erreur PID, sortie PID.
- **Compteur de consolidation** : nombre d'arêtes consolidées affiché en temps réel.
- **Sparkline d'énergie** : 80 derniers ticks.
- **Mode infini** : barre de titre affiche le tick sans borne quand `total_ticks = 0`.
- **Taille des points adaptative** : 4.0px (<500 nœuds), 2.5px (<5 000), 1.5px (≥5 000).

---

## 7. Architecture logicielle V2

### Nouveaux modules

```
stretch-core/src/
├── zone.rs          # NOUVEAU — ZoneManager + Zone + PID + Voronoï
├── pacemaker.rs     # NOUVEAU — oscillateurs sinusoïdaux
├── config.rs        # ÉTENDU — ZoneConfig, ConsolidationConfig, PacemakerConfig
├── edge.rs          # ÉTENDU — consolidation_counter, consolidated
├── simulation.rs    # ÉTENDU — 6 phases, zone_manager, mode infini
├── plasticity.rs    # ÉTENDU — appel update_consolidation()
└── metrics.rs       # ÉTENDU — consolidated_edges, PID metrics
```

### Propriétés conservées

- **Déterminisme** : ChaCha8, seed identique → résultats identiques.
- **Configuration TOML** : aucun paramètre hard-codé, tout est configurable.
- **Modularité** : core / cli / viz — frontends indépendants.

---

## 8. Configurations validées

| Config | Nœuds | Zones | Pacemakers | Consolidation | Usage |
|---|---|---|---|---|---|
| `config_v2_zones.toml` | 50k | 8 PID | 0 | seuil 2.5 | Référence régulation pure |
| `config_v2_pacemakers.toml` | 50k | 8 PID | 3 osc. | seuil 2.5 | Oscillations + régulation |
| `config_v2_training.toml` | 50k | 12 PID | 2 osc. | seuil 2.0 | Entraînement 3 phases |
| `config_v1_*.toml` | 10k | 0 | 0 | non | Rétro-compat V1 |

---

## 9. Résumé des objectifs V2

| Objectif (cahier des charges) | Statut | Remarque |
|---|---|---|
| Activité endogène stable | ✅ | ~9 000 actifs permanents sur 50k |
| Oscillations locales | ✅ | Pacemakers sinusoïdaux |
| Régulation régionale | ✅ | PID par zone Voronoï |
| Mémoire structurelle persistante | ⚠️ | Fonctionne mais non sélective |
| Différenciation neuronale minimale | ✅ | 3 classes : standard, contrôle, pacemaker |
| Rétro-compatibilité V1 | ✅ | Configs V1 inchangées |
| Performance ≥ V1 | ✅ | 50k nœuds, ~21 ms/tick |
| Mode infini | ✅ | total_ticks = 0 |

---

## 10. Dynamiques émergentes observées

### 10.1 Auto-régulation

Le PID crée un régime d'équilibre actif qui n'existait pas en V1. Le système oscille naturellement autour de la consigne avec des fluctuations de ~±500 nœuds actifs, sans jamais diverger ni s'éteindre.

### 10.2 Battements

Quand deux pacemakers à fréquences proches ($f_1 = 0{,}02$, $f_2 = 0{,}025$) sont actifs, on observe des modulations d'amplitude dans le voisinage spatial de leurs zones respectives, avec une fréquence de battement $f_b = |f_1 - f_2| = 0{,}005$ (période ~200 ticks).

### 10.3 Interaction PID / pacemaker

Le PID et les pacemakers forment un système couplé : l'oscillation du pacemaker est une perturbation locale que le PID tente de compenser. Le résultat est une modulation de la sortie PID au rythme du pacemaker — le contrôleur "suit" l'oscillation sans l'étouffer.

### 10.4 Consolidation de masse

Avec le PID maintenant une activité homogène, toutes les arêtes reçoivent de la co-activation et franchissent le seuil de consolidation. C'est un comportement non souhaité : la mémoire structurelle devrait être **sélective** (seuls les chemins réellement sollicités).