# Cahier des charges — V2
## Régulation locale et activité endogène

## 1. Objectif

La V2 vise à résoudre les limitations structurelles identifiées en V1 :

- absence d'équilibre actif
- extinction systématique de l'activité
- oubli structurel des conductances
- absence de régulation locale
- homogénéité des neurones

La V2 introduit un **substrat auto-régulé** capable de maintenir une activité dynamique stable.

---

# 2. Objectifs fonctionnels

La V2 doit permettre :

- activité endogène stable
- oscillations locales
- régulation régionale
- mémoire structurelle persistante
- différenciation neuronale minimale

---

# 3. Types de neurones

La V2 introduit trois classes de neurones :

## 3.1 Neurones standards

Fonction :

propagation et plasticité.

## 3.2 Neurones de contrôle

Fonction :

régulation de l'activité locale.

Ils implémentent un contrôleur :

PID simplifié.

## 3.3 Neurones pacemaker

Fonction :

source d'activité oscillatoire intrinsèque.

---

# 4. Régulation locale

Chaque zone du graphe est régulée par un neurone de contrôle.

Mesure :

activité moyenne de la zone.

Correction :

u = Kp * erreur + Ki * intégrale + Kd * dérivée

---

# 5. Partitionnement spatial

L'espace est divisé en zones.

Méthodes autorisées :

- Voronoï
- rayon fixe
- clustering spatial

Chaque zone possède :

- 1 neurone de contrôle
- N neurones standards

---

# 6. Consolidation mémoire

Les conductances fortement renforcées deviennent persistantes.

Condition :

si

w > w_consolidation

pendant

T_consolidation ticks

alors :

decay désactivé.

---

# 7. Cycle d'un tick V2

Séquence :

1 mesure activité zones  
2 régulation PID  
3 stimulus externe  
4 propagation  
5 dissipation  
6 plasticité  

---

# 8. Critères d'acceptation

La V2 est validée si :

- activité auto-entretenue possible
- oscillations locales observables
- mémoire structurelle persistante
- stabilité globale du système