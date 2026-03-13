# Architecture système — V3

## 1. Objet

Définir l’organisation logicielle et conceptuelle de la V3.

La V3 ajoute trois couches majeures :
- diversité neuronale minimale ;
- plasticité temporelle ;
- contrôle régional indirect.

---

## 2. Architecture logique

```text
Stimuli externes
    ↓
Mesure zone
    ↓
PID indirect
    ↓
Propagation signée E/I
    ↓
Dissipation
    ↓
STDP
    ↓
Sélectivité mémoire / consolidation compétitive
    ↓
Métriques et visualisation
```

---

## 3. Types de nœuds

## 3.1 NodeExcitatory
Rôle :
- propagation positive ;
- apprentissage causal ;
- base des circuits récurrents.

Variables :
- activation
- threshold
- fatigue
- inhibition
- memory_trace
- excitability
- zone_id

## 3.2 NodeInhibitory
Rôle :
- propagation négative ;
- contraste spatial ;
- inhibition latérale ;
- participation aux boucles E/I.

Variables :
- mêmes variables de base ;
- gain inhibiteur dédié ;
- constante de temps inhibitrice potentiellement spécifique.

## 3.3 NodeControl
Rôle :
- mesurer l’activité de la zone ;
- ajuster des paramètres de contexte.

Variables supplémentaires :
- target_activity
- pid_error_prev
- pid_integral
- theta_mod
- gain_mod
- excitability_mod
- budget_mod

## 3.4 NodePacemaker
Rôle :
- amorce de rythmes ;
- tests comparatifs ;
- outil temporaire, importance décroissante.

---

## 4. Arêtes

Les arêtes V3 doivent au minimum porter :

- conductance
- coactivity_trace
- plasticity
- distance
- last_pre_tick
- last_post_tick
- stdp_weight_delta
- consolidated_flag

La structure doit permettre de distinguer :
- renforcement corrélatif éventuel ;
- renforcement causal STDP ;
- mécanismes de consolidation.

---

## 5. Zones

La structure de zone héritée de V2 est conservée mais enrichie.

Chaque zone doit contenir :

- liste des nœuds ;
- nœud de contrôle ;
- activité moyenne ;
- énergie excitatrice ;
- énergie inhibitrice ;
- paramètres modulés (seuil, gain, budget).

---

## 6. Tick V3 recommandé

```text
Phase 0  mesure zones
Phase 1  mise à jour PID indirect
Phase 2  stimulus externe
Phase 3  propagation signée E/I
Phase 4  dissipation
Phase 5  STDP
Phase 6  sélectivité mémoire / consolidation compétitive
Phase 7  métriques
```

---

## 7. Interfaces logicielles recommandées

## 7.1 Core
- `node.rs`
- `edge.rs`
- `zone.rs`
- `propagation.rs`
- `stdp.rs`
- `control.rs`
- `memory_selectivity.rs`
- `simulation.rs`
- `metrics.rs`

## 7.2 CLI
- exécutions batch ;
- sweeps paramétriques ;
- benchmarks.

## 7.3 Viz
- visualisation par type ;
- visualisation E/I ;
- oscillations locales ;
- traces STDP ;
- cartes de sélectivité.

---

## 8. Compatibilité future

L’architecture V3 doit préparer :
- V4 compétition mémoire renforcée ;
- V5 hiérarchie de zones ;
- V6 spécialisation neuronale riche ;
- V7 mémoire multi-système.

Donc :
- ne pas durcir le PID ;
- ne pas figer les types à 2 classes ;
- laisser place à des signaux multiples futurs ;
- garder le partitionnement extensible.
