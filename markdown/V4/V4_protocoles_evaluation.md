# Protocoles d’évaluation — V4

## 1. Objet

Mesurer ce que la V4 apporte réellement par rapport à la V3.

La V4 doit être évaluée comme la première version d’apprentissage guidé.

---

## 2. Axes d’évaluation

### A. Plasticité guidée
- reward change-t-il les poids ?
- reward change-t-il le comportement ?

### B. Entrées
- le système distingue-t-il plusieurs patterns ?

### C. Sorties
- le système produit-il une décision lisible ?

### D. Sélectivité
- la consolidation devient-elle plus utile ?

---

## 3. Évaluations avant/après

## 3.1 Comparaison V3 vs V4

Mesurer :
- distribution des poids ;
- sélectivité mémoire ;
- performance en tâche simple ;
- reward cumulé ;
- contraste entrée/sortie.

Attendu :
- apprentissage orienté par reward ;
- meilleure utilité de la mémoire.

---

## 3.2 Évaluation supervision simple

Procédure :
- deux patterns d’entrée ;
- deux classes de sortie ;
- reward selon la bonne réponse.

Mesures :
- accuracy ;
- temps de convergence ;
- reward cumulé ;
- séparation des sorties.

Attendu :
- accuracy > hasard.

---

## 3.3 Évaluation reward retardé

Procédure :
- délai entre activité utile et reward ;
- variation du délai.

Mesures :
- performance selon délai ;
- sensibilité à `gamma_e`.

Attendu :
- chute progressive avec délai trop long ;
- fenêtre utile non nulle.

---

## 4. Métriques principales

- reward cumulé ;
- accuracy ;
- temps de convergence ;
- entropie des sorties ;
- distribution `e_ij` ;
- Δw rewardés vs non rewardés ;
- nombre d’arêtes consolidées ;
- indice de sélectivité mémoire.
