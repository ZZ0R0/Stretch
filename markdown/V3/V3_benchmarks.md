# Benchmarks — V3

## 1. Objet

Mesurer le coût de la V3 par rapport à la V2.

Les nouveaux postes de coût sont :
- propagation signée E/I ;
- STDP ;
- métriques supplémentaires ;
- éventuelle normalisation synaptique.

---

## 2. Configurations minimales

| Config | Nœuds | Arêtes approx | Usage |
|---|---:|---:|---|
| small_v3 | 10k | dépend topologie | calibration |
| ref_v3 | 50k | dépend topologie | référence |
| stress_v3 | 100k | dépend topologie | projection |

---

## 3. Mesures à collecter

- temps moyen / tick ;
- temps propagation ;
- temps STDP ;
- temps contrôle zone ;
- temps métriques ;
- mémoire RAM ;
- coût visualisation.

---

## 4. Comparaisons obligatoires

- V2 direct PID vs V3 indirect PID ;
- V2 Hebbien corrélatif vs V3 STDP ;
- 0% inhibiteurs vs 20% inhibiteurs ;
- avec / sans sélectivité mémoire.

---

## 5. KPI recommandés

- overhead STDP < 40% à 50k nœuds ;
- overhead PID indirect négligeable face à propagation ;
- maintien sous le seuil d’itération confortable de dev ;
- estimation de passage GPU pour 500k–1M nœuds.

---

## 6. Décision GPU

Un sprint GPU devient prioritaire si :
- la V3 ref_v3 dépasse durablement le budget temps de dev expérimental ;
- les sweeps STDP deviennent trop coûteux ;
- la projection 500k–1M nœuds est irréaliste en CPU pur.
