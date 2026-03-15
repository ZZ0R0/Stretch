# Benchmarks — V5

## 1. Objet

Mesurer le coût de la V5 par rapport à la V4.

Nouveaux postes de coût :
- tâches anti-biais ;
- diagnostics de chemins ;
- analyses de conductance ;
- profils adaptatifs ;
- mécanismes d’activité soutenue.

---

## 2. Configurations minimales

| Config | Nœuds | Arêtes approx | Usage |
|---|---:|---:|---|
| small_v5 | 50k | dépend topologie | debug |
| ref_v5 | 500k | dépend topologie | référence |
| scale_v5 | 5M | dépend topologie | stress-test |

---

## 3. Mesures à collecter

- temps moyen / tick ;
- temps propagation ;
- temps plasticité ;
- temps diagnostics ;
- temps readout ;
- mémoire GPU ;
- mémoire CPU ;
- coût export/visualisation.

---

## 4. Comparaisons obligatoires

- V4 ref vs V5 ref ;
- V5 sans diagnostics vs avec diagnostics ;
- V5 sans sustain vs avec sustain ;
- V5 fixes vs V5 adaptatifs.

---

## 5. KPI recommandés

- conserver un mode expérimental confortable à 50k/500k ;
- supporter les stress-tests à 5M ;
- limiter l’overhead diagnostics hors runs d’analyse ;
- ne pas casser l’avantage GPU acquis en V4.

---

## 6. Décision d’optimisation

Une optimisation supplémentaire est prioritaire si :
- les diagnostics mangent le budget temps ;
- le path tracer devient le goulot ;
- la calibration multi-échelle nécessite trop de sweeps coûteux.
