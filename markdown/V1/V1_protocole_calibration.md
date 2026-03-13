
# Protocole de calibration — V1 espace 3D

## Objectif

Identifier une zone de paramètres stable dans le nouveau domaine 3D.

---

## Paramètres explorés

- propagation_gain
- activation_decay
- fatigue_gain
- inhibition_gain
- radius_neighbors

---

## Méthode

1. générer un graphe spatial 3D
2. stimuler un neurone central
3. observer propagation

---

## Mesures

- nombre de neurones activés
- rayon de propagation
- durée avant extinction
- énergie totale

---

## Résultat attendu

Zone stable :

```
propagation locale
dissipation contrôlée
pas de saturation globale
```
