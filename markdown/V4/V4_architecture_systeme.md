# Architecture système — V4

## 1. Objet

Définir l’organisation logicielle et conceptuelle de la V4.

La V4 ajoute quatre couches majeures :
- modulation dopaminergique ;
- reward externe ;
- traces d’éligibilité ;
- interfaces entrée/sortie minimales.

---

## 2. Architecture logique

```text
Input encoder
    ↓
Zones d’entrée
    ↓
Mesure zones
    ↓
PID indirect
    ↓
Propagation signée E/I
    ↓
Dissipation
    ↓
STDP
    ↓
Eligibility update
    ↓
Reward / dopamine modulation
    ↓
Gating de consolidation
    ↓
Readout sortie
    ↓
Métriques
```

---

## 3. Composants principaux

## 3.1 Core neuronal
Hérite de V3 :
- neurones E ;
- neurones I ;
- neurones de contrôle ;
- neurones pacemaker éventuels.

## 3.2 Module dopaminergique
Nouveau composant responsable de :
- niveau tonique ;
- bursts / dips ;
- latence de modulation ;
- projection vers zones ou globalement.

### Structure possible
```text
DopamineSystem
- tonic_level
- phasic_value
- target_mode (global | zones)
- decay
- gain_plasticity
- gain_consolidation
```

## 3.3 Module eligibility
Nouveau composant responsable de :
- stockage de `e_ij` ;
- update locale à chaque tick ;
- decay ;
- lecture lors de la modulation reward.

### Extension Edge
Ajouter au minimum :
- `eligibility`
- `eligibility_decay`
- `last_rewarded_delta`

## 3.4 Module reward
Composant global simple :
- reward courant ;
- schedule ou feedback online ;
- mode supervisé simple.

### Structure possible
```text
RewardSystem
- current_reward
- reward_schedule
- reward_mode
- cumulative_reward
```

## 3.5 Module input
Encode des patterns externes dans une zone d’entrée.

### Rôle
- transformer un pattern discret en stimulation structurée ;
- isoler clairement l’entrée du reste du graphe.

## 3.6 Module output
Lit l’activité d’une ou plusieurs zones de sortie.

### Rôle
- convertir activité interne → décision ;
- fournir reward ;
- mesurer l’accuracy.

---

## 4. Zones supplémentaires

## 4.1 Zones d’entrée
- dédiées ;
- taille configurable ;
- mapping pattern → activation.

## 4.2 Zones de sortie
- dédiées ;
- groupes de décision ;
- readout winner-take-all ou soft score.

## 4.3 Zones dopaminergiques
Optionnelles en V4 minimal.
Deux options :
- modulation globale ;
- modulation ciblée par zone.

---

## 5. Tick V4 recommandé

```text
Phase 0  encode input
Phase 1  mesure zones
Phase 2  PID indirect
Phase 3  stimulus externe / entrée
Phase 4  propagation signée E/I
Phase 5  dissipation
Phase 6  STDP locale
Phase 7  update eligibility
Phase 8  compute readout sortie
Phase 9  compute reward
Phase 10 dopamine modulation / consolidation gating
Phase 11 métriques
```

---

## 6. Interfaces logicielles recommandées

### Core
- `node.rs`
- `edge.rs`
- `zone.rs`
- `propagation.rs`
- `stdp.rs`
- `eligibility.rs`
- `dopamine.rs`
- `reward.rs`
- `input.rs`
- `output.rs`
- `simulation.rs`
- `metrics.rs`

### CLI
- batch learning ;
- sweeps reward/eligibility ;
- comparaison STDP vs reward-STDP.

### Viz
- cartes input/output ;
- reward cumulé ;
- dopamine ;
- distributions d’éligibilité ;
- poids récompensés.

---

## 7. Compatibilité future

L’architecture V4 doit préparer :
- V5 mémoire sélective guidée ;
- V6 hiérarchie de zones ;
- V7 spécialisation riche ;
- V8 assemblées et mémoire de travail.

Il faut donc :
- éviter que dopamine soit un hack local ;
- isoler proprement input/output ;
- garder `eligibility` extensible ;
- laisser place à plusieurs canaux modulateurs futurs.
