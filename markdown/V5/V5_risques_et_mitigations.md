# Risques et mitigations — V5

## 1. Objet

Identifier les risques majeurs de la V5 et définir les mitigations ainsi que les gates de passage vers la V6.

---

## 2. Risques majeurs

## 2.1 Le réseau n’apprend toujours pas
Risque :
- la topologie continue d’expliquer les résultats ;
- la plasticité n’apporte pas de gain réel.

Mitigation :
- tâches anti-biais dès le début ;
- baselines strictes ;
- arrêt précoce si aucune tâche n’est battue.

## 2.2 Les chemins restent diffus
Risque :
- beaucoup d’arêtes changent faiblement sans route claire.

Mitigation :
- path tracer ;
- métriques CT ;
- renforcement focalisé ;
- tests de re-routing.

## 2.3 La dynamique soutenue déstabilise le réseau
Risque :
- emballement ;
- oscillations non voulues ;
- perte de contrôle.

Mitigation :
- sweeps progressifs ;
- clamps ;
- profils adaptatifs bornés ;
- comparaison systématique avec V4.

## 2.4 La calibration multi-échelle n’a pas de loi simple
Risque :
- chaque taille exige des paramètres totalement différents.

Mitigation :
- profils de calibration explicites ;
- grid search automatisés ;
- dashboards de comparaison ;
- accepter des lois approchées si elles restent utiles.

## 2.5 Les diagnostics coûtent trop cher
Risque :
- impossible de faire des sweeps complets.

Mitigation :
- diagnostics activables séparément ;
- modes light vs heavy ;
- exports différés.

---

## 3. Gate de sortie V5

Passage à V6 autorisé uniquement si :

1. une tâche anti-biais est apprise ;
2. full learning bat topology-only ;
3. les chemins renforcés sont cohérents ;
4. la dynamique soutenue progresse sans instabilité critique ;
5. la calibration multi-échelle améliore au moins partiellement le passage 50k → 500k → 5M.

---

## 4. Si le gate échoue

- si l’apprentissage n’est pas prouvé : itération V5 centrée preuve ;
- si la preuve existe mais la dynamique reste flash & die : itération V5 centrée sustain ;
- si tout marche sauf à grande échelle : itération V5 centrée calibration ;
- si les diagnostics sont trop lourds : optimisation outil avant V6.
