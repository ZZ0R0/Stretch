# Benchmarks — V4

## 1. Objet

Mesurer le coût de la V4 par rapport à la V3.

Nouveaux postes de coût :
- update eligibility ;
- dopamine/reward ;
- I/O ;
- readout ;
- éventuelle asymétrie STDP.

---

## 2. Configurations minimales

| Config | Nœuds | Arêtes approx | Usage |
|---|---:|---:|---|
| small_v4 | 10k | dépend topologie | calibration |
| ref_v4 | 50k | dépend topologie | référence |
| scale_v4 | 500k | dépend topologie | projection |

---

## 3. Mesures à collecter

- temps moyen / tick ;
- temps propagation ;
- temps STDP ;
- temps eligibility ;
- temps dopamine/reward ;
- temps readout sortie ;
- temps I/O ;
- mémoire RAM ;
- overhead viz.

---

## 4. Comparaisons obligatoires

- V3 full vs V4 sans reward ;
- V4 sans reward vs V4 reward ;
- V4 sans eligibility vs avec eligibility ;
- V4 sans I/O vs avec I/O.

---

## 5. KPI recommandés

- overhead eligibility + reward raisonnable face à propagation+STDP ;
- coût readout négligeable à 50k ;
- coût I/O maîtrisé ;
- estimation crédible de portage GPU pour 500k–1M nœuds.

---

## 6. Décision GPU

Le sprint GPU devient prioritaire si :
- la V4 ref_v4 ne permet plus de sweeps confortables ;
- la V4 scale_v4 est inutilisable avec instrumentation complète ;
- le readout ou la viz deviennent le goulot dominant.
