# V3 — Index documentaire

## Objet

Ce dossier regroupe les documents nécessaires pour concevoir, implémenter, calibrer et valider la **V3** du projet Stretch.

La V3 constitue la transition critique entre :

- un système **maintenu par contrôle externe** ;
- et un système dont les dynamiques commencent à émerger de l’interaction entre types neuronaux.

---

## Base acquise avant V3

- **V0** : propagation, dissipation, traces, plasticité locale.
- **V1** : substrat 3D non-grid, KD-tree, topologies KNN/radius, calibration 3D, 50k nœuds, formalisation mathématique complète. 
- **V2** : zones, PID, pacemakers, activité auto-entretenue, mémoire structurelle durable. Cependant la V2 présente encore :
  - consolidation de masse ;
  - homogénéité PID ;
  - absence d’inhibition inter-neuronale ;
  - plasticité non causale ;
  - absence d’assemblées stables et de compétition. 

---

## Objectif central de la V3

Transformer le système de :

```text
substrat irrigué par un contrôleur
→
substrat dont les dynamiques émergent de l’interaction entre types neuronaux
```

---

## Fichiers du lot V3

- `V3_cahier_des_charges.md`
- `V3_architecture_systeme.md`
- `V3_modelisation_mathematique.md`
- `V3_protocoles_stabilite.md`
- `V3_protocoles_evaluation.md`
- `V3_protocoles_STDP.md`
- `V3_protocoles_oscillations_emergentes.md`
- `V3_benchmarks.md`
- `V3_risques_et_gates.md`
- `V3_aboutissement.md`

---

## Piliers structurants de la V3

1. **Inhibition inter-neuronale**  
   ~20% de neurones inhibiteurs, création de contraste spatial, compétition locale, oscillations E/I.

2. **STDP**  
   Plasticité dépendante du timing, apprentissage de séquences causales, différenciation A→B / B→A.

3. **PID indirect**  
   Le PID n’injecte plus directement l’activation ; il ajuste les conditions-cadre du réseau.

4. **Première sélectivité mémoire**  
   Réduction de la consolidation de masse par normalisation, seuils, gating et compétition.

---

## Gate de sortie V3

La V3 n’est validée que si :

- au moins une oscillation émergente existe sans pacemaker ;
- un apprentissage séquentiel directionnel est démontré ;
- la mémoire structurelle devient plus sélective qu’en V2 ;
- l’activité auto-entretenue subsiste alors que le PID n’agit plus directement sur `activation`.
