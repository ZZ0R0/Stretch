# Cahier des charges — V5
## Preuve d’apprentissage réelle, calibration multi-échelle, dynamique soutenue

## 1. Objet

La V5 a pour objectif de transformer la V4 d’une infrastructure d’apprentissage scalable en un système dont l’apprentissage est **scientifiquement démontré**.

Le problème central n’est plus l’infrastructure.
Le problème central est :

> le réseau apprend-il réellement, ou la topologie résout-elle encore la tâche à sa place ?

---

## 2. Objectifs fonctionnels

La V5 doit permettre :

- des tâches sans biais topologique ;
- des baselines strictes de comparaison ;
- une calibration multi-échelle ;
- une dynamique d’activité plus soutenue ;
- des outils diagnostics lisibles des chemins et conductances ;
- une comparaison quantitative entre apprentissage et simple géométrie.

---

## 3. Périmètre V5

### Inclus
- redesign des tâches d’évaluation ;
- configurations I/O symétriques et anti-biais ;
- re-routing et inversion de mapping ;
- calibration paramétrique selon l’échelle ;
- mécanismes de dynamique soutenue ;
- outils de lecture des chemins ;
- métriques de preuve d’apprentissage ;
- benchmarks 50k / 500k / 5M.

### Exclus
- hiérarchie complète micro/méso/macro ;
- nouveaux types neuronaux riches ;
- assemblées explicites ;
- symbolisation ;
- texte ;
- refonte totale de l’architecture GPU.

---

## 4. Exigences fonctionnelles

## 4.1 Tâches anti-biais topologique

La V5 doit intégrer au moins trois familles de tâches :

### A. Configuration symétrique
Les distances entre entrées et sorties concurrentes doivent être rendues comparables.

### B. Association inversée
Exemple :
- input-0 doit apprendre à activer output-1, même si output-0 est plus proche géométriquement.

### C. Re-apprentissage
Le système doit pouvoir :
- apprendre une première association ;
- puis l’inverser ;
- puis mesurer le temps de ré-apprentissage.

---

## 4.2 Baselines obligatoires

La V5 doit permettre la comparaison entre :

- **random baseline** : poids aléatoires, pas d’apprentissage ;
- **topology-only baseline** : géométrie active, plasticité désactivée ;
- **full learning** : plasticité active.

Au moins une tâche V5 doit être évaluée sur ces trois régimes.

---

## 4.3 Calibration multi-échelle

Les hyperparamètres ne peuvent plus être purement fixes.

La V5 doit permettre des lois adaptatives selon :
- nombre de nœuds `n` ;
- extent spatial ;
- group_size ;
- read_delay ;
- densité de connexions ;
- taille des groupes d’entrée/sortie.

Paramètres candidats à rendre adaptatifs :
- propagation gain ;
- eligibility decay ;
- conductance decay ;
- spatial lambda ;
- target_activity ;
- presentation_ticks.

---

## 4.4 Dynamique soutenue

La V5 doit casser le régime strictement impulsionnel V4.

Mécanismes autorisés :
- decay adaptatif ;
- maintien local dépendant de l’activité ;
- récurrence locale contrôlée ;
- durée de présentation plus longue ;
- réverbération locale ;
- réduction des resets destructeurs.

La V5 n’a pas besoin d’atteindre une dynamique parfaitement continue, mais elle doit être moins proche d’un cycle :
injection → pic → chute → silence.

---

## 4.5 Outils diagnostics

La V5 doit fournir au minimum :

- carte de conductance 3D ;
- path tracer input→output ;
- timeline eligibility / conductance ;
- analyse de clusters de co-renforcement ;
- comparaison visuelle baseline vs learning.

---

## 4.6 Métriques de preuve

La V5 doit mesurer au minimum :

- accuracy ;
- reward cumulé ;
- conductance directionnelle ;
- cohérence topologique des chemins ;
- gain learning vs topology-only ;
- temps de ré-apprentissage ;
- ratio activité soutenue / activité impulsionnelle ;
- robustesse 50k / 500k / 5M.

---

## 5. Exigences non fonctionnelles

- maintien du mode GPU-first ;
- déterminisme à seed fixe ;
- configurations entièrement externes ;
- répétabilité des protocoles ;
- instrumentation exportable ;
- coût compatible avec sweeps paramétriques.

---

## 6. Livrables obligatoires

- moteur V5 ;
- configs anti-biais ;
- configs baseline ;
- configs calibration multi-échelle ;
- outils diagnostics ;
- benchmarks ;
- note d’aboutissement V5.

---

## 7. Critères d’acceptation

La V5 est validée si :

1. au moins une tâche anti-biais est apprise ;
2. la baseline topology-only est battue ;
3. des routes renforcées cohérentes sont observées ;
4. l’activité est plus soutenue qu’en V4 ;
5. la calibration multi-échelle améliore le comportement au-delà de 50k ;
6. le protocole de preuve d’apprentissage est archivé, reproductible et interprétable.
