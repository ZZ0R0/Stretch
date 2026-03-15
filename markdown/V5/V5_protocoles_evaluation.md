# Protocoles d’évaluation — V5

## 1. Objet

Mesurer ce que la V5 apporte réellement par rapport à la V4.

La V5 doit être évaluée comme une version de preuve d’apprentissage.

---

## 2. Axes d’évaluation

### A. Preuve d’apprentissage
- le modèle bat-il la géométrie seule ?

### B. Formation de chemins
- les conductances forment-elles des routes utiles ?

### C. Dynamique
- l’activité devient-elle plus soutenue ?

### D. Scaling
- les paramètres s’adaptent-ils à l’échelle ?

---

## 3. Comparaisons obligatoires

## 3.1 Random baseline
Poids aléatoires, pas d’apprentissage.

## 3.2 Topology-only baseline
Topologie réelle, plasticité off.

## 3.3 Full learning
Plasticité complète.

---

## 4. Tâches d’évaluation

### A. Symmetric IO
Distances équivalentes.

### B. Inverted mapping
Association contraire à la géométrie intuitive.

### C. Re-learning
Inversion après pré-entraînement.

---

## 5. Métriques principales

- accuracy ;
- reward cumulé ;
- RouteScore ;
- indice directionnel ;
- cohérence topologique CT ;
- sustain ratio ;
- temps de convergence ;
- temps de ré-apprentissage ;
- sensibilité à l’échelle.

---

## 6. Condition de réussite

La V5 est considérée comme réussie si :
- full learning > topology-only sur au moins une tâche anti-biais ;
- full learning > hasard ;
- les chemins appris sont cohérents visuellement et quantitativement ;
- la dynamique soutenue s’améliore ;
- les résultats restent valides au moins sur deux échelles de taille.
