# V5 — Index documentaire

## Objet

Ce dossier regroupe les documents nécessaires pour concevoir, implémenter, calibrer et valider la **V5** du projet Stretch.

La V5 n’est pas une simple montée de complexité cognitive.
C’est la première version dont l’objectif principal est de :

> **prouver que le réseau apprend réellement au-delà du biais topologique**

Elle doit aussi :
- stabiliser une dynamique plus soutenue ;
- fournir une calibration multi-échelle ;
- introduire de vrais outils diagnostics.

---

## Base acquise avant V5

- **V0** : propagation, dissipation, traces, plasticité locale.
- **V1** : substrat 3D non-grid, KD-tree, topologies KNN/radius, calibration 3D.
- **V2** : activité auto-entretenue régulée, partitionnement spatial, consolidation.
- **V3** : E/I, propagation signée, PID indirect, STDP, budget synaptique compétitif.
- **V4** : architecture GPU-first, pipeline complet STDP → éligibilité → 3 facteurs → homéostasie → consolidation, I/O minimales, scalabilité 5M nœuds.

---

## Pourquoi la V5 existe

La V4 a validé l’infrastructure, mais pas encore l’apprentissage réel.

Points bloquants hérités de V4 :
- accuracy 100% expliquée par la géométrie ;
- régime flash & die ;
- chemins dopaminergiques non prouvés ;
- éligibilité trop courte ;
- modifications de conductance trop faibles ;
- paramètres non robustes à l’échelle.

La V5 doit donc :
- prouver ;
- calibrer ;
- diagnostiquer ;
- stabiliser.

---

## Axes structurants de la V5

1. **Tâches anti-biais topologique**
2. **Baselines strictes**
3. **Calibration multi-échelle**
4. **Dynamique soutenue**
5. **Outils diagnostics des chemins**

---

## Fichiers du lot V5

- `V5_README.md`
- `V5_cahier_des_charges.md`
- `V5_architecture_systeme.md`
- `V5_modelisation_mathematique.md`
- `V5_protocoles_stabilite.md`
- `V5_protocoles_evaluation.md`
- `V5_protocoles_preuve_apprentissage.md`
- `V5_protocoles_calibration_multiechelle.md`
- `V5_benchmarks.md`
- `V5_risques_et_mitigations.md`
- `V5_aboutissement.md`

---

## Gate de sortie V5

La V5 n’est validée que si :

- une tâche anti-biais est apprise au-dessus du hasard ;
- la baseline topologie-only est battue ;
- des routes input→output cohérentes sont observées ;
- l’activité est moins pulsée et plus soutenue ;
- la calibration multi-échelle améliore réellement le passage 50k → 500k → 5M.
