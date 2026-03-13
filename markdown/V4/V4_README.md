# V4 — Index documentaire

## Objet

Ce dossier regroupe les documents nécessaires pour concevoir, implémenter, calibrer et valider la **V4** du projet Stretch.

La V4 est la première version qui transforme Stretch de :

```text
substrat auto-organisé
→
substrat capable d'apprendre en fonction d'un objectif
```

Elle introduit les briques manquantes identifiées après V3 :

- voies dopaminergiques minimales ;
- signal de récompense externe ;
- traces d’éligibilité ;
- interface d’entrée minimale ;
- interface de sortie minimale.

---

## Base acquise avant V4

- **V0** : propagation, dissipation, traces, plasticité locale.
- **V1** : substrat 3D non-grid, KD-tree, topologies KNN/radius, calibration 3D, 50k nœuds, formalisation mathématique.
- **V2** : partitionnement spatial, PID, pacemakers, activité auto-entretenue, consolidation structurelle.
- **V3** : E/I, propagation signée, PID indirect, STDP, budget synaptique compétitif, 500k nœuds CPU, substrat moins homogène. 

---

## Verrous post-V3 traités par la V4

- pas de dopamine ;
- pas de reward ;
- pas de traces d’éligibilité ;
- pas d’entrée structurée ;
- pas de sortie interprétable ;
- STDP encore trop symétrique ;
- consolidation encore trop rapide sous fort fond d’activité. 

---

## Objectif central de la V4

Construire un système qui peut :

1. recevoir un pattern structuré ;
2. produire une sortie lisible ;
3. recevoir une récompense ou une punition ;
4. modifier sa plasticité en conséquence ;
5. apprendre plus ce qui est récompensé que ce qui est simplement fréquent.

---

## Fichiers du lot V4

- `V4_README.md`
- `V4_cahier_des_charges.md`
- `V4_architecture_systeme.md`
- `V4_modelisation_mathematique.md`
- `V4_protocoles_stabilite.md`
- `V4_protocoles_evaluation.md`
- `V4_protocoles_reward_eligibility.md`
- `V4_protocoles_io_minimales.md`
- `V4_benchmarks.md`
- `V4_risques_et_gates.md`
- `V4_aboutissement.md`

---

## Gate de sortie V4

La V4 n’est validée que si :

- le reward change effectivement les poids appris ;
- les traces d’éligibilité permettent un apprentissage à récompense retardée ;
- les patterns d’entrée sont distingués spatialement ;
- la sortie permet un readout au-dessus du hasard ;
- la dopamine agit comme modulateur de plasticité, pas comme nouveau contrôleur caché.
