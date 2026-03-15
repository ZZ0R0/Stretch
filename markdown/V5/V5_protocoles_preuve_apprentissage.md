# Protocoles de preuve d’apprentissage — V5

## 1. Objet

Valider que Stretch apprend réellement et ne se contente pas d’exploiter la géométrie.

---

## 2. Hypothèse centrale

Si la topologie seule ne suffit pas à résoudre une tâche, alors un réseau avec plasticité active doit surperformer :
- la baseline aléatoire ;
- la baseline topologie-only.

---

## 3. Expériences

## 3.1 Configuration symétrique

Placer :
- inputs et outputs à distances comparables.

Mesures :
- accuracy ;
- RouteScore ;
- CT.

Attendu :
- la baseline topologie-only retombe proche du hasard ;
- full learning la dépasse.

---

## 3.2 Association inversée

Conserver une géométrie connue mais imposer :
- input-0 → output-1 ;
- input-1 → output-0.

Mesures :
- vitesse d’apprentissage ;
- évolution des routes ;
- performances.

Attendu :
- re-routing observable.

---

## 3.3 Re-apprentissage

Procédure :
1. apprendre mapping A ;
2. inverser ;
3. mesurer le temps de convergence vers B.

Mesures :
- temps de ré-apprentissage ;
- héritage des anciennes routes ;
- compétition entre anciennes et nouvelles routes.

---

## 4. Critères de réussite

- une tâche anti-biais est apprise ;
- full learning > topology-only ;
- les routes changent dans le bon sens ;
- la preuve est reproductible sur plusieurs seeds.
