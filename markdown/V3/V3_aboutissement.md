# Aboutissement V3

## Résumé

La V3 accomplit la transition décrite dans la vision post-V2 : le système passe d'un **substrat irrigué par un contrôleur** à un **substrat dont les dynamiques émergent de l'interaction entre types neuronaux**. Les quatre piliers structurants ont été implémentés et validés : inhibition E/I, STDP, PID indirect, et budget synaptique. Le moteur a été parallélisé pour supporter 500k nœuds sur 16 threads CPU.

---

## 1. Bilan des objectifs

| Objectif (cahier des charges V3) | Statut | Détail |
|---|---|---|
| Neurones excitateurs / inhibiteurs | ✅ | 80% E / 20% I, proportion configurable |
| Propagation signée | ✅ | Le signe du signal dépend du type source |
| PID indirect | ✅ | Ajuste θ_mod et gain_mod, plus d'injection directe |
| STDP | ✅ | LTP/LTD avec fenêtres exponentielles symétriques |
| Sélectivité mémoire | ✅ | Budget synaptique par nœud source (quota sortant) |
| Oscillations émergentes | ⚠️ | Non validées formellement par protocole d'ablation |
| Métriques E/I | ✅ | Actifs E/I, énergie E/I séparées |
| Performance ≥50k nœuds | ✅ | 50k : ~17.7 ms/tick ; 500k : fonctionnel |
| Déterminisme à graine fixe | ✅ | Conservé (ChaCha8 pour assignment, hash pour jitter) |
| Rétro-compatibilité V2 | ✅ | Toutes les extensions optionnelles (enabled = false) |

---

## 2. Pilier 1 — Inhibition inter-neuronale

### Implémentation

- Chaque nœud possède un `NeuronType` : `Excitatory` ou `Inhibitory`.
- L'assignation est aléatoire et reproductible (ChaCha8, seed + 77777).
- Cible : 20% d'inhibiteurs (configurable via `inhibitory_fraction`).

### Propagation signée

Le signal émis par un nœud inhibiteur est multiplié par $-g_I$ :

$$\text{contrib}_i = a_i \times \begin{cases} +1 & \text{si } i \in E \\ -g_I & \text{si } i \in I \end{cases} \times g_{\text{global}} \times (1 + g_{\text{zone}})$$

Avec $g_I = 0{,}8$ par défaut. Le signal inhibiteur **réduit** l'activation des cibles au lieu de l'augmenter.

### Résultat observé

- L'activité n'est plus spatialement homogène : les neurones inhibiteurs créent des **trous** dans les patterns d'activation.
- Les zones proches d'amas I ont une activité plus basse, nécessitant une compensation PID locale.
- Le contraste spatial est nettement supérieur à V2.

### Instabilité V2 résolue

- ✅ **I5 — Pas d'inhibition** : résolu. 20% de neurones inhibiteurs avec propagation signée.
- ✅ **I2 — Homogénéité PID** : partiellement résolu. L'inhibition brise l'uniformité spatiale.

---

## 3. Pilier 2 — STDP

### Implémentation

Chaque nœud stocke son `last_activation_tick`. À chaque tick, pour chaque arête $(i, j)$ :

$$\Delta t = t_{\text{post}}^j - t_{\text{pré}}^i$$

$$\Delta w = \begin{cases} +A_+ \exp\!\left(-\frac{\Delta t}{\tau_+}\right) & \text{si } \Delta t > 0 \text{ (LTP)} \\ -A_- \exp\!\left(\frac{\Delta t}{\tau_-}\right) & \text{si } \Delta t < 0 \text{ (LTD)} \end{cases}$$

Paramètres validés :

| Paramètre | Symbole | Valeur |
|---|---|---|
| Amplitude LTP | $A_+$ | 0.005 |
| Amplitude LTD | $A_-$ | 0.005 |
| Constante LTP | $\tau_+$ | 20.0 ticks |
| Constante LTD | $\tau_-$ | 20.0 ticks |

### Résultat observé

- Le profil est **symétrique** (même amplitude LTP/LTD, mêmes constantes de temps). C'est un choix conservateur de V3.
- La STDP coexiste avec la plasticité Hebbienne dans une passe fusionnée unique sur toutes les arêtes.
- Les conductances sont bornées par $[w_{\min}, w_{\max}]$ après chaque application.
- La directionnalité est mesurable : un stimulus A suivi de B renforce A→B plus que B→A.

### Instabilité V2 résolue

- ✅ **I6 — Plasticité temporellement aveugle** : résolu. La STDP différencie l'ordre pré→post et post→pré.

---

## 4. Pilier 3 — PID indirect

### Implémentation

Le PID ne modifie plus directement l'activation des nœuds. Il agit sur deux leviers :

$$\theta_{\text{mod}} = -k_\theta \times u(t)$$
$$g_{\text{mod}} = +k_g \times u(t)$$

où $u(t)$ est la sortie PID classique (P + I + D avec anti-windup).

Effets :
- **Activité trop basse** ($u > 0$) → seuil effectif baisse ($\theta_{\text{mod}} < 0$) → les neurones tirent plus facilement ; gain augmente → signaux plus forts.
- **Activité trop haute** ($u < 0$) → seuil monte ; gain baisse.

Le seuil effectif de chaque nœud intègre la modulation :

$$\theta_{\text{eff}} = \frac{\theta + f + h + \theta_{\text{mod}}}{\varepsilon}$$

### Paramètres

| Paramètre | Symbole | Valeur |
|---|---|---|
| Mode | `pid_mode` | `"indirect"` |
| Gain seuil | $k_\theta$ | 0.3 |
| Gain propagation | $k_g$ | 0.2 |

### Résultat observé

- Le réseau **se maintient par propagation récurrente**, le PID ne faisant qu'ajuster les conditions-cadre.
- La dépendance au PID est réduite : la coupure du PID n'éteint plus le réseau instantanément (vs ~20 ticks en V2).
- Le PID indirect est plus lent à stabiliser que le direct, mais produit une activité plus **naturelle** : l'énergie résulte de la propagation entre nœuds, pas d'une injection externe.

### Instabilité V2 résolue

- ✅ **I3 — PID omnipotent** : résolu. Le PID n'injecte plus d'activation directement.

---

## 5. Pilier 4 — Budget synaptique

### Implémentation

Pour chaque nœud source $i$, si la somme des conductances sortantes dépasse le budget :

$$W_i = \sum_{j \in \text{adj}(i)} w_{ij}$$

$$\text{si } W_i > B \text{ et } \neg \text{consolidated}_{ij} : \quad w_{ij} \leftarrow \max\!\left(w_{\min},\; w_{ij} \times \frac{B}{W_i}\right)$$

Les arêtes consolidées sont **exemptées** : elles ne sont ni réduites ni comptabilisées dans la normalisation.

### Paramètres

| Paramètre | Valeur |
|---|---|
| Budget $B$ | 30.0 |

### Résultat observé

- La consolidation de masse est **fortement réduite** par rapport à V2.
- Les arêtes sont en compétition pour un budget fini : le renforcement d'une connexion implique l'affaiblissement relatif des autres connexions sortantes du même nœud.
- Couplé à l'inhibition (qui crée du contraste), seuls les chemins réellement sollicités se renforcent et se consolident.

### Instabilité V2 résolue

- ✅ **I1 — Consolidation de masse** : résolu. Le budget contraint les conductances sortantes par nœud.

---

## 6. Optimisations de performance

### Parallélisation rayon

| Phase | Parallélisation |
|---|---|
| Zone.measure() | `par_iter_mut` sur les zones |
| Propagation: source contribs | `par_iter` sur les nœuds |
| Propagation: accumulation | `par_iter` sur `incoming_adjacency` |
| Propagation: application | `par_iter_mut` sur les nœuds |
| Dissipation | `par_iter_mut` sur les nœuds |
| PLAST + STDP + BUD | `par_iter_mut` sur les arêtes |
| Métriques | `par_fold` / `par_reduce` |

### Auto-détection des threads

Le pool rayon détecte automatiquement le nombre de cœurs disponibles via `std::thread::available_parallelism()` et configure le pool global au lancement.

### Passe fusionnée PLAST+STDP+BUD

Trois opérations autrefois séparées (plasticité Hebbienne, STDP, normalisation budget) sont exécutées dans une **unique** itération parallèle sur les ~577k arêtes (50k) ou ~5M arêtes (500k). Cela réduit les passes de 4 à 2 par tick (propagation + fused), minimisant les défauts de cache L3 sur les ~37 MB de données d'arêtes.

### Jitter déterministe sans RNG

Le jitter de dissipation utilise un hash rapide au lieu de ChaCha8Rng :

$$h = ((i \oplus t) \times 6364136223846793005 + 1442695040888963407) \gg 33$$

Zéro allocation, zéro synchronisation inter-threads.

### Rendu viz optimisé

Les 50k+ appels `draw_circle` (20+ vertices chacun) ont été remplacés par `draw_rectangle` (4 vertices), réduisant la charge GPU d'un facteur ~5x.

### Benchmarks

| Configuration | Nœuds | Arêtes | ms/tick | Threads |
|---|---|---|---|---|
| V2 ref (avant optim) | 50k | 577k | ~21 | 1 |
| V3 ref (après optim) | 50k | 577k | ~17.7 | 16 |
| V3 full | 500k | ~5M | fonctionnel | 16 |

---

## 7. Cycle de simulation V3 — 8 phases

```
Phase 0 : Mesure          — activité moyenne par zone (parallèle)
Phase 1 : Régulation      — PID indirect : ajuste θ_mod et gain_mod
Phase 2 : Stimulus        — injections externes + pacemakers
Phase 3 : Propagation     — signée E/I, target-centric (parallèle)
Phase 4 : Dissipation     — decay, fatigue, inhibition, trace, jitter (parallèle)
Phase 5-7 : PLAST+STDP+BUD — passe unique fusionnée (parallèle)
Phase 8 : Métriques       — snapshot tous les N ticks (parallèle)
```

---

## 8. Configurations validées V3

| Config | Nœuds | E/I | STDP | PID | Budget | Usage |
|---|---|---|---|---|---|---|
| `config_v3_ei.toml` | 50k | ✅ 20% I | ❌ | direct | ❌ | Test inhibition isolée |
| `config_v3_stdp.toml` | 50k | ✅ | ✅ | direct | ❌ | Test STDP isolé |
| `config_v3_full.toml` | 500k | ✅ 20% I | ✅ | indirect | ✅ 30.0 | Référence V3 complète |

---

## 9. Gains par rapport à V2

| Dimension | V2 | V3 |
|---|---|---|
| Types neuronaux | Aucun (tous identiques) | Excitatory / Inhibitory |
| Propagation | Toujours positive | Signée (E > 0, I < 0) |
| PID | Direct (injection a) | Indirect (θ_mod, gain_mod) |
| Plasticité | Hebbienne corrélative | Hebbienne + STDP temporelle |
| Mémoire | Consolidation de masse | Budget synaptique compétitif |
| Contraste spatial | Homogène | Hétérogène (inhibition locale) |
| Parallélisme | Mono-thread | 16 threads (rayon auto-detect) |
| Performance 50k | ~21 ms/tick (1 CPU) | ~17.7 ms/tick (16 CPU) |
| Scale max testée | 50k nœuds | 500k nœuds |

---

## 10. Gate de sortie V3 — Évaluation

| Critère de gate | Résultat | Détail |
|---|---|---|
| Inhibition fonctionnelle | ✅ | 20% I, propagation signée, contraste spatial |
| STDP directionnelle démontrée | ✅ | LTP/LTD avec fenêtres temporelles, coexistence Hebbian |
| PID réellement indirect | ✅ | θ_mod + gain_mod, aucune injection directe |
| Oscillation émergente mesurée | ⚠️ | Non validée par protocole formel d'ablation sans pacemaker |
| Sélectivité mémoire > V2 | ✅ | Budget synaptique empêche la consolidation de masse |

**Décision** : 4 critères sur 5 atteints. Le critère oscillatoire nécessite un protocole de validation dédié (ablation pacemaker + analyse spectrale). Le passage à V4 peut être envisagé, conditionné à une itération V3.x si les oscillations émergentes ne se confirment pas.

---

## 11. Instabilités résiduelles

Voir `V3_instabilites.md` pour le détail complet. En résumé :

- oscillations émergentes non formellement validées ;
- consolidation encore trop rapide sous forte activité PID ;
- asymétrie E/I potentiellement sous-exploitée (profil STDP symétrique) ;
- scalabilité GPU non adressée (500k fonctionne mais lent pour le temps réel) ;
- absence de voies dopaminergiques et de système de récompense.
