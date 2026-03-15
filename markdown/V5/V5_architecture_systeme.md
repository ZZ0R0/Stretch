# Architecture système — V5

## 1. Objet

Définir l’organisation logicielle et conceptuelle de la V5.

La V5 ne change pas fondamentalement l’architecture GPU-first de la V4.
Elle ajoute surtout :

- des tâches et configurations plus rigoureuses ;
- des mécanismes de dynamique soutenue ;
- des outils diagnostics ;
- une couche de calibration multi-échelle.

---

## 2. Architecture logique

```text
Config task
    ↓
Input placement / output placement
    ↓
Task encoder
    ↓
Propagation GPU
    ↓
Plasticité GPU
    ↓
Eligibility + dopamine + reward
    ↓
Readout sortie
    ↓
Task evaluator
    ↓
Diagnostic layer
    ↓
Calibration / benchmark layer
```

---

## 3. Composants principaux

## 3.1 TaskSystem
Responsable de :
- définition des patterns d’entrée ;
- définition des cibles ;
- inversion de mapping ;
- séquences de trials ;
- comparaisons baseline / learning.

### Sous-modes recommandés
- `symmetric_io`
- `inverted_mapping`
- `remap_after_pretrain`
- `topology_only`
- `random_baseline`
- `full_learning`

---

## 3.2 CalibrationSystem
Responsable de :
- lois adaptatives d’hyperparamètres ;
- scaling selon `n` ;
- configuration de presentation_ticks ;
- calibration eligibility/read_delay ;
- calibration decay/activity.

### Structure possible
```text
CalibrationProfile
- gain_rule
- eligibility_rule
- edge_decay_rule
- lambda_rule
- target_activity_rule
- presentation_ticks_rule
```

---

## 3.3 SustainedActivitySystem
Responsable de :
- decay adaptatif ;
- réverbération locale ;
- récurrence locale contrôlée ;
- maintien partiel d’activité.

### Variables candidates
- `activation_decay_local`
- `local_feedback_gain`
- `reverberation_gain`
- `reset_policy`

---

## 3.4 DiagnosticsSystem
Responsable de :
- calcul de chemins conductifs ;
- cartes de chaleur ;
- timeline eligibility/conductance ;
- clusters de co-renforcement ;
- comparaison baseline vs learning.

### Sorties attendues
- JSON/CSV métriques
- snapshots 3D
- traces temporelles
- score de cohérence des chemins

---

## 4. Tick V5 recommandé

```text
Phase 0  encode task input
Phase 1  injection / entrée
Phase 2  propagation
Phase 3  dissipation + dynamique soutenue
Phase 4  STDP locale
Phase 5  update eligibility
Phase 6  reward / dopamine modulation
Phase 7  readout sortie
Phase 8  task evaluation
Phase 9  diagnostics
```

---

## 5. Interfaces logicielles recommandées

### Core
- `task.rs`
- `calibration.rs`
- `sustained_activity.rs`
- `diagnostics.rs`
- `path_analysis.rs`
- `cluster_analysis.rs`

### CLI
- sweeps paramétriques ;
- runs baseline ;
- comparaison tasks.

### Viz
- conductance 3D ;
- chemins ;
- activité soutenue ;
- comparaison par overlays.

---

## 6. Compatibilité future

La V5 doit préparer :
- V6 chemins dopaminergiques robustes ;
- V7 hiérarchie ;
- V8 spécialisation riche ;
- V9 assemblées.

Il faut donc :
- garder la task layer séparée du moteur ;
- garder les diagnostics réutilisables ;
- ne pas coder les lois adaptatives en dur.
