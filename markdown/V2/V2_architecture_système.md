# Architecture système — V2

## 1. Objectif

Définir l'organisation logicielle et les nouveaux composants introduits en V2.

---

# 2. Types de noeuds

## NodeStandard

Rôle :

propagation et plasticité.

Variables :

activation  
fatigue  
inhibition  
trace mémoire  
excitabilité  

---

## NodeControl

Rôle :

régulation d'une zone.

Variables supplémentaires :

zone_activity_mean  
target_activity  
pid_integral  
pid_error_prev  

---

## NodePacemaker

Rôle :

génération d'oscillation interne.

Equation :

a(t+1) = a(t) + A * sin(2πft)

---

# 3. Zones

Structure :

Zone

Contient :

- liste de noeuds
- neurone de contrôle

---

# 4. Graphe spatial

Structure héritée de V1 :

- graphe spatial 3D
- index KD-tree

---

# 5. Cycle simulation

```

measure_zones()

control_update()

stimulus_injection()

propagation()

dissipation()

plasticity()

```

---

# 6. Extensions futures

L'architecture doit permettre :

- spécialisation neuronale
- hiérarchie de zones
- mémoire multi-niveaux